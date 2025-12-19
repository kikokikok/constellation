//! Simple placeholder for multi-dimensional indexing

use super::{MemoryError, Result};

/// Multi-dimensional index
pub struct MultiDimensionalIndex;

impl Default for MultiDimensionalIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiDimensionalIndex {
    pub fn new() -> Self {
        Self
    }

    pub fn index(
        &mut self,
        _working_id: super::WorkingMemoryId,
        _episodic_id: super::EpisodicMemoryId,
        _vector_id: super::VectorId,
        _context: &super::MemoryContext,
        _modality: super::MemoryModality,
        _granularity: super::Granularity,
    ) -> Result<()> {
        Ok(())
    }
}
