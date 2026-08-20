//! Multi-tier memory subsystem (L1 working memory and L2 episodic session store).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// L1 in-context working memory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L1WorkingMemory {
    pub session_id: String,
    pub goal: String,
    pub pinned_system_prompt: String,
    pub working_variables: HashMap<String, String>,
    pub recent_steps: Vec<String>,
    pub max_step_history: usize,
}

impl L1WorkingMemory {
    pub fn new(session_id: impl Into<String>, goal: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            goal: goal.into(),
            pinned_system_prompt: String::new(),
            working_variables: HashMap::new(),
            recent_steps: Vec::new(),
            max_step_history: 50,
        }
    }

    /// Pushes a step record into working memory.
    pub fn record_step(&mut self, step_summary: impl Into<String>) {
        if self.recent_steps.len() >= self.max_step_history {
            self.recent_steps.remove(0);
        }
        self.recent_steps.push(step_summary.into());
    }

    /// Sets a working variable.
    pub fn set_var(&mut self, key: impl Into<String>, val: impl Into<String>) {
        self.working_variables.insert(key.into(), val.into());
    }

    /// Retrieves a working variable.
    pub fn get_var(&self, key: &str) -> Option<&String> {
        self.working_variables.get(key)
    }
}

/// An episodic session record in L2 memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L2EpisodicRecord {
    pub session_id: String,
    pub timestamp: u64,
    pub goal: String,
    pub total_steps: usize,
    pub final_summary: String,
    pub success: bool,
    pub artifacts_produced: Vec<String>,
}

/// L2 Episodic Session Store.
#[derive(Debug, Clone, Default)]
pub struct L2EpisodicStore {
    records: HashMap<String, L2EpisodicRecord>,
}

impl L2EpisodicStore {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    /// Commits a completed session to episodic memory.
    pub fn commit_session(&mut self, record: L2EpisodicRecord) {
        self.records.insert(record.session_id.clone(), record);
    }

    /// Retrieves an episodic record by session ID.
    pub fn get_session(&self, session_id: &str) -> Option<&L2EpisodicRecord> {
        self.records.get(session_id)
    }

    /// Returns the total number of stored episodes.
    pub fn session_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_l1_l2_lifecycle() {
        let mut l1 = L1WorkingMemory::new("sess-1", "Build Xeno");
        l1.record_step("Step 1: Init project");
        l1.record_step("Step 2: Implement core");
        assert_eq!(l1.recent_steps.len(), 2);

        let mut l2 = L2EpisodicStore::new();
        l2.commit_session(L2EpisodicRecord {
            session_id: "sess-1".into(),
            timestamp: 1771580400000,
            goal: "Build Xeno".into(),
            total_steps: 2,
            final_summary: "Core built".into(),
            success: true,
            artifacts_produced: vec!["crates/xeno-core".into()],
        });

        assert_eq!(l2.session_count(), 1);
        let rec = l2.get_session("sess-1").unwrap();
        assert!(rec.success);
    }
}
