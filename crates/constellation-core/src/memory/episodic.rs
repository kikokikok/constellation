//! Episodic memory for events and experiences

use crate::memory::context::MemoryContext;
use crate::memory::MemoryModality;

/// Episodic memory trace
#[derive(Debug, Clone)]
pub struct MemoryTrace {
    pub content: String,
    pub context: MemoryContext,
    pub modality: MemoryModality,
    pub timestamp: std::time::Instant,
}

impl MemoryTrace {
    pub fn new(content: String, context: MemoryContext, modality: MemoryModality) -> Self {
        Self {
            content,
            context,
            modality,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn update(&self, new_content: String, context: MemoryContext) -> Self {
        Self {
            content: new_content,
            context,
            modality: self.modality,
            timestamp: std::time::Instant::now(),
        }
    }
}

/// Episodic memory system
#[derive(Debug, Default)]
pub struct EpisodicMemory {
    traces: std::collections::HashMap<EpisodicMemoryId, MemoryTrace>,
}

impl EpisodicMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, trace: MemoryTrace) -> Result<EpisodicMemoryId, crate::memory::MemoryError> {
        let id = EpisodicMemoryId::new();
        self.traces.insert(id, trace);
        Ok(id)
    }

    pub fn retrieve_trace(&self, id: EpisodicMemoryId) -> Result<MemoryTrace, crate::memory::MemoryError> {
        self.traces.get(&id).cloned().ok_or_else(|| crate::memory::MemoryError::EpisodicMemory("Trace not found".to_string()))
    }

    pub fn update(&mut self, trace: MemoryTrace) -> Result<(), crate::memory::MemoryError> {
        // For now, just record as new
        self.record(trace)?;
        Ok(())
    }
}

/// Episodic memory ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EpisodicMemoryId(u64);

impl EpisodicMemoryId {
    fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}