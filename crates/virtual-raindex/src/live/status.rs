use std::fmt;

use serde::{Deserialize, Serialize};

use super::sync_engine::ArtifactId;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LivePhase {
    Idle,
    Syncing,
    PendingArtifacts,
    Errored,
}

impl Default for LivePhase {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PendingArtifacts {
    pub artifacts: Vec<ArtifactId>,
}

impl PendingArtifacts {
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LiveStatus {
    pub phase: LivePhase,
    pub pending: PendingArtifacts,
    pub advisories: Vec<LiveAdvisory>,
}

impl LiveStatus {
    pub fn with_phase(phase: LivePhase) -> Self {
        Self {
            phase,
            ..Self::default()
        }
    }

    pub fn with_pending(artifacts: Vec<ArtifactId>) -> Self {
        Self {
            phase: LivePhase::PendingArtifacts,
            pending: PendingArtifacts { artifacts },
            advisories: vec![LiveAdvisory::WarmingUp],
        }
    }

    pub fn idle() -> Self {
        Self {
            phase: LivePhase::Idle,
            pending: PendingArtifacts::default(),
            advisories: vec![LiveAdvisory::Ready],
        }
    }

    pub fn syncing_ready() -> Self {
        Self {
            phase: LivePhase::Syncing,
            pending: PendingArtifacts::default(),
            advisories: vec![LiveAdvisory::Ready],
        }
    }

    pub fn errored(reason: impl Into<String>) -> Self {
        Self {
            phase: LivePhase::Errored,
            pending: PendingArtifacts::default(),
            advisories: vec![LiveAdvisory::Degraded {
                reason: reason.into(),
            }],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LiveAdvisory {
    WarmingUp,
    Ready,
    Degraded { reason: String },
}

impl fmt::Display for LiveAdvisory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiveAdvisory::WarmingUp => write!(f, "warming up"),
            LiveAdvisory::Ready => write!(f, "ready"),
            LiveAdvisory::Degraded { reason } => write!(f, "degraded: {reason}"),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SyncProgress {
    pub applied_batches: usize,
    pub mutation_count: usize,
    pub cached_artifacts: usize,
    pub deferred_mutations: usize,
}
