use std::{collections::VecDeque, fs, path::Path, sync::Arc};

use alloy::{hex, primitives::Address};
use parking_lot::RwLock;

use crate::{
    error::{RaindexError, Result},
    state::RaindexMutation,
    BytecodeKind,
};
use rain_orderbook_bindings::IOrderBookV5::OrderV4;
use serde::Deserialize;

use super::sync_engine::{
    ArtifactBatch, ArtifactId, BytecodeArtifact, MutationEnvelope, SyncEngine, SyncPoll,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StubCursor(pub u64);

impl From<u64> for StubCursor {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Default)]
pub struct StubStep {
    pub next_cursor: Option<StubCursor>,
    pub mutation_batches: Vec<MutationEnvelope>,
    pub bytecode: ArtifactBatch,
    pub pending_artifacts: Vec<ArtifactId>,
    pub heartbeat: bool,
}

impl StubStep {
    pub fn with_mutations(mut self, envelope: MutationEnvelope) -> Self {
        self.mutation_batches.push(envelope);
        self
    }

    pub fn with_bytecode(mut self, artifact: BytecodeArtifact) -> Self {
        self.bytecode.push(artifact);
        self
    }

    pub fn with_pending(mut self, artifact: ArtifactId) -> Self {
        self.pending_artifacts.push(artifact);
        self
    }

    pub fn heartbeat(mut self) -> Self {
        self.heartbeat = true;
        self
    }
}

#[derive(Clone, Default)]
pub struct StubSyncEngine {
    steps: Arc<RwLock<VecDeque<StubStep>>>,
}

impl StubSyncEngine {
    pub fn new() -> Self {
        Self {
            steps: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub fn with_steps<I>(steps: I) -> Self
    where
        I: IntoIterator<Item = StubStep>,
    {
        let mut queue = VecDeque::new();
        for step in steps {
            queue.push_back(step);
        }
        Self {
            steps: Arc::new(RwLock::new(queue)),
        }
    }

    pub fn from_json_str(data: &str) -> Result<Self> {
        let configs: Vec<StubStepConfig> = serde_json::from_str(data)
            .map_err(|_| RaindexError::Unimplemented("failed to parse stub script json"))?;
        let mut steps = Vec::with_capacity(configs.len());
        for config in configs {
            steps.push(config.into_step()?);
        }
        Ok(Self::with_steps(steps))
    }

    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self> {
        let data = fs::read_to_string(path.as_ref())
            .map_err(|_| RaindexError::Unimplemented("failed to read stub script from disk"))?;
        Self::from_json_str(&data)
    }

    pub fn push_step(&self, step: StubStep) {
        self.steps.write().push_back(step);
    }

    pub fn remaining(&self) -> usize {
        self.steps.read().len()
    }

    pub fn demo() -> Self {
        Self::from_json_str(include_str!(
            "../../../../test-resources/virtual_raindex/live/demo.json"
        ))
        .expect("demo script fixture must be valid")
    }
}

#[async_trait::async_trait]
impl SyncEngine for StubSyncEngine {
    type Cursor = StubCursor;

    async fn poll(
        &self,
        _orderbook: Address,
        cursor: Option<Self::Cursor>,
    ) -> Result<SyncPoll<Self::Cursor>> {
        let mut guard = self.steps.write();
        if let Some(step) = guard.pop_front() {
            Ok(SyncPoll {
                next_cursor: step.next_cursor.or(cursor),
                mutation_batches: step.mutation_batches,
                bytecode: step.bytecode,
                pending_artifacts: step.pending_artifacts,
                heartbeat: step.heartbeat,
            })
        } else {
            Ok(SyncPoll {
                next_cursor: cursor,
                mutation_batches: Vec::new(),
                bytecode: Vec::new(),
                pending_artifacts: Vec::new(),
                heartbeat: true,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
struct StubStepConfig {
    #[serde(default)]
    cursor: Option<u64>,
    #[serde(default)]
    env: Option<StubEnvConfig>,
    #[serde(default)]
    orders: Vec<String>,
    #[serde(default)]
    bytecode: Vec<StubArtifactConfig>,
    #[serde(default)]
    pending: Vec<StubArtifactConfig>,
    #[serde(default)]
    heartbeat: bool,
}

#[derive(Debug, Deserialize)]
struct StubEnvConfig {
    #[serde(default)]
    block_number: Option<u64>,
    #[serde(default)]
    timestamp: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum StubArtifactConfig {
    Interpreter {
        address: Address,
        #[serde(default)]
        bytecode: Option<String>,
    },
    Store {
        address: Address,
        #[serde(default)]
        bytecode: Option<String>,
    },
}

impl StubStepConfig {
    fn into_step(self) -> Result<StubStep> {
        let mut step = StubStep::default();
        if let Some(cursor) = self.cursor {
            step.next_cursor = Some(StubCursor(cursor));
        }

        if let Some(env) = self.env {
            step = step.with_mutations(MutationEnvelope::new(vec![RaindexMutation::SetEnv {
                block_number: env.block_number,
                timestamp: env.timestamp,
            }]));
        }

        if !self.orders.is_empty() {
            let mut orders = Vec::with_capacity(self.orders.len());
            for entry in self.orders {
                let order = match entry.as_str() {
                    "default" => OrderV4::default(),
                    _ => {
                        return Err(RaindexError::Unimplemented("unknown stub order fixture"));
                    }
                };
                orders.push(order);
            }
            step = step.with_mutations(MutationEnvelope::new(vec![RaindexMutation::SetOrders {
                orders,
            }]));
        }

        if !self.bytecode.is_empty() {
            for artifact in self.bytecode {
                step = step.with_bytecode(artifact.to_bytecode()?);
            }
        }

        if !self.pending.is_empty() {
            for artifact in self.pending {
                step = step.with_pending(artifact.artifact_id());
            }
        }

        if self.heartbeat {
            step.heartbeat = true;
        }

        Ok(step)
    }
}

impl StubArtifactConfig {
    fn artifact_id(&self) -> ArtifactId {
        match self {
            StubArtifactConfig::Interpreter { address, .. } => {
                ArtifactId::new(*address, BytecodeKind::Interpreter)
            }
            StubArtifactConfig::Store { address, .. } => {
                ArtifactId::new(*address, BytecodeKind::Store)
            }
        }
    }

    fn to_bytecode(&self) -> Result<BytecodeArtifact> {
        let bytes = decode_artifact_bytes(match self {
            StubArtifactConfig::Interpreter { bytecode, .. } => bytecode.as_ref(),
            StubArtifactConfig::Store { bytecode, .. } => bytecode.as_ref(),
        })?;

        Ok(match self {
            StubArtifactConfig::Interpreter { address, .. } => {
                BytecodeArtifact::interpreter(*address, bytes)
            }
            StubArtifactConfig::Store { address, .. } => BytecodeArtifact::store(*address, bytes),
        })
    }
}

fn decode_artifact_bytes(raw: Option<&String>) -> Result<Vec<u8>> {
    let bytes = if let Some(value) = raw {
        let trimmed = value.trim();
        let data = trimmed.strip_prefix("0x").unwrap_or(trimmed);
        let decoded = hex::decode(data)
            .map_err(|_| RaindexError::Unimplemented("invalid stub bytecode encoding"))?;
        if decoded.is_empty() {
            default_bytecode()
        } else {
            decoded
        }
    } else {
        default_bytecode()
    };

    Ok(bytes)
}

fn default_bytecode() -> Vec<u8> {
    vec![0x60, 0x00, 0x60, 0x00]
}
