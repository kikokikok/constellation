//! Session management for long-running agent state

use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{Error, Result};

/// Session state
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SessionState {
    /// Session is active and running
    Active,
    /// Session is paused
    Paused,
    /// Session is completed
    Completed,
    /// Session has failed
    Failed(String),
}

/// Agent session
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Session {
    /// Unique session ID
    pub id: String,
    /// Agent ID
    pub agent_id: String,
    /// Current state
    pub state: SessionState,
    /// Session data (serialized state)
    pub data: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Timeout duration
    pub timeout: StdDuration,
}

impl Session {
    /// Create a new session
    pub fn new(agent_id: String, data: String, timeout: StdDuration) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            state: SessionState::Active,
            data,
            created_at: now,
            updated_at: now,
            timeout,
        }
    }

    /// Update session data
    pub fn update(&mut self, data: String) {
        self.data = data;
        self.updated_at = Utc::now();
    }

    /// Change session state
    pub fn set_state(&mut self, state: SessionState) {
        self.state = state;
        self.updated_at = Utc::now();
    }

    /// Check if session has expired
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let elapsed = now - self.updated_at;
        elapsed > Duration::from_std(self.timeout).unwrap_or(Duration::seconds(0))
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Active)
    }
}

/// Session manager for storing and retrieving sessions
#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<std::collections::HashMap<String, Session>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        agent_id: String,
        data: String,
        timeout: StdDuration,
    ) -> Result<String> {
        let session = Session::new(agent_id, data, timeout);
        let session_id = session.id.clone();

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<Session> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| Error::SessionNotFound(session_id.to_string()))
    }

    /// Update session data
    pub async fn update_session(&self, session_id: &str, data: String) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.update(data);
            Ok(())
        } else {
            Err(Error::SessionNotFound(session_id.to_string()))
        }
    }

    /// Update session state
    pub async fn update_session_state(&self, session_id: &str, state: SessionState) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.set_state(state);
            Ok(())
        } else {
            Err(Error::SessionNotFound(session_id.to_string()))
        }
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut sessions = self.sessions.write().await;
        let initial_len = sessions.len();

        sessions.retain(|_, session| !session.is_expired());

        let removed = initial_len - sessions.len();
        Ok(removed)
    }

    /// Get all sessions for an agent
    pub async fn get_agent_sessions(&self, agent_id: &str) -> Result<Vec<Session>> {
        let sessions = self.sessions.read().await;
        let agent_sessions: Vec<Session> = sessions
            .values()
            .filter(|session| session.agent_id == agent_id)
            .cloned()
            .collect();

        Ok(agent_sessions)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
