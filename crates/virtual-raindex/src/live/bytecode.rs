use std::collections::HashSet;

use super::sync_engine::{ArtifactId, MutationEnvelope};

#[derive(Clone, Debug)]
pub(crate) struct PendingMutation {
    pub envelope: MutationEnvelope,
    pub required: Vec<ArtifactId>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BytecodeWarmupQueue {
    pending: Vec<PendingMutation>,
}

impl BytecodeWarmupQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue(&mut self, envelope: MutationEnvelope, required: Vec<ArtifactId>) {
        self.pending.push(PendingMutation { envelope, required });
    }

    pub fn take_ready<F>(&mut self, mut is_ready: F) -> Vec<MutationEnvelope>
    where
        F: FnMut(&ArtifactId) -> bool,
    {
        let mut ready = Vec::new();
        let mut remaining = Vec::with_capacity(self.pending.len());

        for pending in self.pending.drain(..) {
            if pending.required.iter().all(&mut is_ready) {
                ready.push(pending.envelope);
            } else {
                remaining.push(pending);
            }
        }

        self.pending = remaining;
        ready
    }

    pub fn pending_artifacts(&self) -> Vec<ArtifactId> {
        let mut dedup = HashSet::new();
        let mut artifacts = Vec::new();
        for pending in &self.pending {
            for artifact in &pending.required {
                if dedup.insert(*artifact) {
                    artifacts.push(*artifact);
                }
            }
        }
        artifacts
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }
}
