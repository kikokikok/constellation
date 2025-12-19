//! Simple placeholder for memory consolidation

use super::{MemoryError, Result};

/// Consolidation stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsolidationStage {
    Encoding,
    Synaptic,
    Systems,
    Consolidated,
    Reconsolidating,
}

/// Consolidation pipeline
pub struct ConsolidationPipeline;

impl Default for ConsolidationPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsolidationPipeline {
    pub fn new() -> Self {
        Self
    }

    pub fn schedule(
        &mut self,
        _working_id: super::WorkingMemoryId,
        _stage: ConsolidationStage,
    ) -> Result<()> {
        Ok(())
    }

    pub fn process(&mut self) -> Result<super::ConsolidationReport> {
        Ok(super::ConsolidationReport {
            synaptic_consolidated: 0,
            systems_consolidated: 0,
            reconsolidated: 0,
            forgotten: 0,
            duration_ms: 0,
        })
    }

    pub fn reconsolidate(&mut self, _working_id: super::WorkingMemoryId) -> Result<()> {
        Ok(())
    }
}
