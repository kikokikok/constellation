use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: FeatureStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub git_commit: Option<String>,
    pub test_results: Option<TestResults>,
    pub notes: Vec<String>,
    pub dependencies: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub passed: bool,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub duration_seconds: f64,
    pub screenshots: Vec<PathBuf>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub agent_type: String,
    pub features_worked_on: Vec<Uuid>,
    pub git_commits: Vec<String>,
    pub summary: String,
    pub context_used: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressFile {
    pub project_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub features: Vec<Feature>,
    pub sessions: Vec<Session>,
    pub current_session: Option<Uuid>,
    pub git_branch: String,
    pub last_commit: String,
    pub environment_setup: bool,
    pub next_steps: Vec<String>,
    pub blockers: Vec<String>,
}

pub struct ProgressTracker {
    path: PathBuf,
    progress: ProgressFile,
}

impl ProgressTracker {
    pub fn new(project_path: &Path) -> Result<Self, Error> {
        let progress_path = project_path.join("claude-progress.txt");

        if progress_path.exists() {
            let content = fs::read_to_string(&progress_path)?;
            let progress: ProgressFile = serde_json::from_str(&content)
                .map_err(|e| Error::InvalidProgressFile(e.to_string()))?;

            Ok(Self {
                path: progress_path,
                progress,
            })
        } else {
            let progress = ProgressFile {
                project_name: project_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                features: Vec::new(),
                sessions: Vec::new(),
                current_session: None,
                git_branch: "main".to_string(),
                last_commit: String::new(),
                environment_setup: false,
                next_steps: Vec::new(),
                blockers: Vec::new(),
            };

            Ok(Self {
                path: progress_path,
                progress,
            })
        }
    }

    pub fn add_feature(&mut self, name: &str, description: &str) -> Uuid {
        let feature = Feature {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            status: FeatureStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            git_commit: None,
            test_results: None,
            notes: Vec::new(),
            dependencies: Vec::new(),
        };

        self.progress.features.push(feature.clone());
        self.save().expect("Failed to save progress");
        feature.id
    }

    pub fn update_feature_status(
        &mut self,
        feature_id: Uuid,
        status: FeatureStatus,
    ) -> Result<(), Error> {
        let feature = self
            .progress
            .features
            .iter_mut()
            .find(|f| f.id == feature_id)
            .ok_or_else(|| Error::FeatureNotFound(feature_id.to_string()))?;

        let status_clone = status.clone();
        feature.status = status;
        feature.updated_at = Utc::now();

        if matches!(status_clone, FeatureStatus::Completed) {
            feature.completed_at = Some(Utc::now());
        }

        self.save()
    }

    pub fn start_session(&mut self, agent_type: &str) -> Uuid {
        let session = Session {
            id: Uuid::new_v4(),
            started_at: Utc::now(),
            ended_at: None,
            agent_type: agent_type.to_string(),
            features_worked_on: Vec::new(),
            git_commits: Vec::new(),
            summary: String::new(),
            context_used: 0,
            errors: Vec::new(),
        };

        self.progress.sessions.push(session.clone());
        self.progress.current_session = Some(session.id);
        self.save().expect("Failed to save progress");
        session.id
    }

    pub fn end_session(
        &mut self,
        session_id: Uuid,
        summary: &str,
        context_used: usize,
    ) -> Result<(), Error> {
        let session = self
            .progress
            .sessions
            .iter_mut()
            .find(|s| s.id == session_id)
            .ok_or_else(|| Error::InvalidState("Session not found".to_string()))?;

        session.ended_at = Some(Utc::now());
        session.summary = summary.to_string();
        session.context_used = context_used;

        self.progress.current_session = None;
        self.save()
    }

    pub fn add_git_commit(&mut self, commit_hash: &str) -> Result<(), Error> {
        self.progress.last_commit = commit_hash.to_string();

        if let Some(session_id) = self.progress.current_session {
            let session = self
                .progress
                .sessions
                .iter_mut()
                .find(|s| s.id == session_id)
                .ok_or_else(|| Error::InvalidState("Session not found".to_string()))?;

            session.git_commits.push(commit_hash.to_string());
        }

        self.save()
    }

    pub fn get_next_feature(&self) -> Option<&Feature> {
        self.progress
            .features
            .iter()
            .find(|f| matches!(f.status, FeatureStatus::Pending))
    }

    pub fn get_in_progress_feature(&self) -> Option<&Feature> {
        self.progress
            .features
            .iter()
            .find(|f| matches!(f.status, FeatureStatus::InProgress))
    }

    pub fn save(&mut self) -> Result<(), Error> {
        self.progress.updated_at = Utc::now();
        let content = serde_json::to_string_pretty(&self.progress)?;
        fs::write(&self.path, content)?;
        Ok(())
    }

    pub fn compress(&mut self) -> Result<(), Error> {
        let max_sessions = 10;
        if self.progress.sessions.len() > max_sessions {
            self.progress.sessions = self
                .progress
                .sessions
                .split_off(self.progress.sessions.len() - max_sessions);
        }

        self.save()
    }

    pub fn get_summary(&self) -> String {
        let completed = self
            .progress
            .features
            .iter()
            .filter(|f| matches!(f.status, FeatureStatus::Completed))
            .count();

        let total = self.progress.features.len();
        let progress = if total > 0 {
            (completed as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        format!(
            "Project: {}\nProgress: {:.1}% ({} of {} features)\nLast commit: {}\nBlockers: {}",
            self.progress.project_name,
            progress,
            completed,
            total,
            self.progress.last_commit,
            self.progress.blockers.len()
        )
    }
}
