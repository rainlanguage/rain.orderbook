use std::fmt::Debug;

use alloy::primitives::Address;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{error::Result, state::RaindexMutation, BytecodeKind};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ArtifactId {
    pub address: Address,
    pub kind: BytecodeKind,
}

impl ArtifactId {
    pub const fn new(address: Address, kind: BytecodeKind) -> Self {
        Self { address, kind }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BytecodeArtifact {
    pub artifact: ArtifactId,
    pub bytecode: Vec<u8>,
}

impl BytecodeArtifact {
    pub fn interpreter(address: Address, bytecode: Vec<u8>) -> Self {
        Self {
            artifact: ArtifactId::new(address, BytecodeKind::Interpreter),
            bytecode,
        }
    }

    pub fn store(address: Address, bytecode: Vec<u8>) -> Self {
        Self {
            artifact: ArtifactId::new(address, BytecodeKind::Store),
            bytecode,
        }
    }
}

pub type ArtifactBatch = Vec<BytecodeArtifact>;

#[derive(Clone, Debug, Default)]
pub struct MutationEnvelope {
    pub mutations: Vec<RaindexMutation>,
}

impl MutationEnvelope {
    pub fn new(mutations: Vec<RaindexMutation>) -> Self {
        Self { mutations }
    }

    pub fn is_empty(&self) -> bool {
        self.mutations.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct SyncPoll<C>
where
    C: Clone + Send + Sync + Debug + Eq + PartialEq + 'static,
{
    pub next_cursor: Option<C>,
    pub mutation_batches: Vec<MutationEnvelope>,
    pub bytecode: ArtifactBatch,
    pub pending_artifacts: Vec<ArtifactId>,
    pub heartbeat: bool,
}

#[async_trait]
pub trait SyncEngine: Send + Sync {
    type Cursor: Clone + Send + Sync + Debug + Eq + PartialEq + 'static;

    async fn poll(
        &self,
        orderbook: Address,
        cursor: Option<Self::Cursor>,
    ) -> Result<SyncPoll<Self::Cursor>>;
}
