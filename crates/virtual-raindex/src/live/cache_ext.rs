use crate::{
    cache::{CodeCache, StaticCodeCache},
    error::Result,
};

use super::sync_engine::{ArtifactId, BytecodeArtifact};

pub trait LiveCodeCache: CodeCache {
    fn ingest(&self, artifact: &BytecodeArtifact) -> Result<()>;
    fn is_ready(&self, artifact: &ArtifactId) -> bool;
}

impl LiveCodeCache for StaticCodeCache {
    fn ingest(&self, artifact: &BytecodeArtifact) -> Result<()> {
        match artifact.artifact.kind {
            crate::BytecodeKind::Interpreter => {
                self.upsert_interpreter(artifact.artifact.address, &artifact.bytecode)
            }
            crate::BytecodeKind::Store => {
                self.upsert_store(artifact.artifact.address, &artifact.bytecode)
            }
        }
    }

    fn is_ready(&self, artifact: &ArtifactId) -> bool {
        match artifact.kind {
            crate::BytecodeKind::Interpreter => self.interpreter(artifact.address).is_some(),
            crate::BytecodeKind::Store => self.store(artifact.address).is_some(),
        }
    }
}
