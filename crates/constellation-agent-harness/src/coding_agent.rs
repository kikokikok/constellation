use std::path::{Path, PathBuf};
use std::fs;
use crate::error::{Error, Result};
use crate::progress::{ProgressTracker, Feature, FeatureStatus};
use crate::git::{GitManager, RepositoryState};
use crate::testing::TestingIntegration;
use crate::memory::MemoryManager;
use indicatif::{ProgressBar, ProgressStyle};
use uuid::Uuid;

pub struct CodingAgent {
    project_path: PathBuf,
    progress_tracker: ProgressTracker,
    git_manager: GitManager,
    testing: TestingIntegration,
    memory: MemoryManager,
    current_session_id: Option<Uuid>,
    context_budget: usize,
    context_used: usize,
}

impl CodingAgent {
    pub fn new(project_path: &Path, context_budget: usize) -> Result<Self> {
        let git_manager = GitManager::open(project_path)?;
        let progress_tracker = ProgressTracker::new(project_path)?;
        let testing = TestingIntegration::new(project_path)?;
        let memory = MemoryManager::new(project_path)?;
        
        Ok(Self {
            project_path: project_path.to_path_buf(),
            progress_tracker,
            git_manager,
            testing,
            memory,
            current_session_id: None,
            context_budget,
            context_used: 0,
        })
    }

    pub fn start_session(&mut self) -> Result<Uuid> {
        if self.current_session_id.is_some() {
            return Err(Error::InvalidState("Session already in progress".to_string()));
        }
        
        self.ensure_clean_state()?;
        
        let session_id = self.progress_tracker.start_session("coding_agent");
        self.current_session_id = Some(session_id);
        
        self.context_used = 0;
        
        Ok(session_id)
    }

    pub fn end_session(&mut self, summary: &str) -> Result<()> {
        let session_id = self.current_session_id
            .ok_or_else(|| Error::InvalidState("No active session".to_string()))?;
        
        self.ensure_clean_state()?;
        
        self.progress_tracker.end_session(session_id, summary, self.context_used)?;
        self.current_session_id = None;
        
        self.progress_tracker.compress()?;
        
        Ok(())
    }

    pub fn select_next_feature(&mut self) -> Result<Option<Feature>> {
        if let Some(feature) = self.progress_tracker.get_in_progress_feature() {
            return Ok(Some(feature.clone()));
        }
        
        if let Some(feature) = self.progress_tracker.get_next_feature() {
            self.progress_tracker.update_feature_status(
                feature.id,
                FeatureStatus::InProgress
            )?;
            
            return Ok(Some(feature.clone()));
        }
        
        Ok(None)
    }

    pub fn implement_feature(&mut self, feature: &Feature) -> Result<bool> {
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>3}% {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        
        pb.set_message(format!("Implementing: {}", feature.name));
        
        self.context_used += 100;
        self.check_context_limit()?;
        
        pb.inc(10);
        
        let implementation_result = self.write_feature_code(feature)?;
        if !implementation_result {
            pb.finish_with_message("Feature implementation failed");
            self.progress_tracker.update_feature_status(
                feature.id,
                FeatureStatus::Failed
            )?;
            return Ok(false);
        }
        
        pb.inc(40);
        
        self.git_manager.add_all()?;
        let commit_message = format!("feat: implement {}", feature.name);
        let commit_hash = self.git_manager.commit(
            &commit_message,
            "Coding Agent",
            "coding@constellation.dev"
        )?;
        
        self.progress_tracker.add_git_commit(&commit_hash)?;
        
        pb.inc(20);
        
        let test_result = self.testing.run_feature_tests(feature)?;
        if !test_result.passed {
            pb.finish_with_message("Tests failed");
            self.handle_test_failure(feature, &test_result)?;
            return Ok(false);
        }
        
        pb.inc(20);
        
        self.progress_tracker.update_feature_status(
            feature.id,
            FeatureStatus::Completed
        )?;
        
        pb.inc(10);
        pb.finish_with_message("Feature completed successfully");
        
        Ok(true)
    }

    fn write_feature_code(&self, feature: &Feature) -> Result<bool> {
        let feature_dir = self.project_path.join("src").join(&feature.name);
        if !feature_dir.exists() {
            fs::create_dir_all(&feature_dir)?;
        }
        
        let lib_path = feature_dir.join("lib.rs");
        let lib_content = format!(
            "//! {}\n//!\n//! {}\n\npub mod implementation;\npub mod tests;\n",
            feature.name, feature.description
        );
        
        fs::write(lib_path, lib_content)?;
        
        let impl_path = feature_dir.join("implementation.rs");
        let impl_content = format!(
            "// Implementation for: {}\n\npub struct {} {{}}\n\nimpl {} {{
    pub fn new() -> Self {{
        Self {{}}
    }}
    
    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {{
        // TODO: Implement feature logic
        Ok(())
    }}
}}",
            feature.name,
            feature.name.replace(" ", "").replace(":", ""),
            feature.name.replace(" ", "").replace(":", "")
        );
        
        fs::write(impl_path, impl_content)?;
        
        let test_path = feature_dir.join("tests.rs");
        let test_content = format!(
            "#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn test_{}_creation() {{
        let feature = {}::new();
        assert!(true); // Placeholder test
    }}
    
    #[test]
    fn test_{}_execution() {{
        let feature = {}::new();
        let result = feature.execute();
        assert!(result.is_ok());
    }}
}}",
            feature.name.to_lowercase().replace(" ", "_").replace(":", ""),
            feature.name.replace(" ", "").replace(":", ""),
            feature.name.to_lowercase().replace(" ", "_").replace(":", ""),
            feature.name.replace(" ", "").replace(":", "")
        );
        
        fs::write(test_path, test_content)?;
        
        Ok(true)
    }

    fn handle_test_failure(&mut self, feature: &Feature, test_result: &crate::testing::TestResult) -> Result<()> {
        self.progress_tracker.update_feature_status(
            feature.id,
            FeatureStatus::Blocked
        )?;
        
        let error_message = test_result.error_message
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Unknown test failure");
        
        self.git_manager.add_all()?;
        let commit_message = format!("fix: attempt to fix test failures for {}", feature.name);
        self.git_manager.commit(
            &commit_message,
            "Coding Agent",
            "coding@constellation.dev"
        )?;
        
        Err(Error::TestingFailed(error_message.to_string()))
    }

    pub fn recover_from_broken_state(&mut self) -> Result<()> {
        let git_state = self.git_manager.get_state()?;
        
        if git_state.has_uncommitted_changes {
            self.git_manager.revert_to_last_good_state()?;
        }
        
        if let Some(feature) = self.progress_tracker.get_in_progress_feature() {
            self.progress_tracker.update_feature_status(
                feature.id,
                FeatureStatus::Pending
            )?;
        }
        
        Ok(())
    }

    fn ensure_clean_state(&self) -> Result<()> {
        let git_state = self.git_manager.get_state()?;
        
        if git_state.has_uncommitted_changes {
            return Err(Error::InvalidState(
                "There are uncommitted changes. Please commit or stash before starting a new session.".to_string()
            ));
        }
        
        Ok(())
    }

    fn check_context_limit(&self) -> Result<()> {
        if self.context_used > self.context_budget {
            return Err(Error::ContextWindowLimitExceeded);
        }
        
        Ok(())
    }

    pub fn compress_context(&mut self) -> Result<String> {
        let summary = self.memory.compress_session(
            self.current_session_id
                .ok_or_else(|| Error::InvalidState("No active session".to_string()))?,
            &self.progress_tracker.progress
        )?;
        
        self.context_used = summary.len() / 4;
        
        Ok(summary)
    }

    pub fn get_session_summary(&self) -> Result<String> {
        let session_id = self.current_session_id
            .ok_or_else(|| Error::InvalidState("No active session".to_string()))?;
        
        let session = self.progress_tracker.progress.sessions
            .iter()
            .find(|s| s.id == session_id)
            .ok_or_else(|| Error::InvalidState("Session not found".to_string()))?;
        
        let features_completed = self.progress_tracker.progress.features
            .iter()
            .filter(|f| matches!(f.status, FeatureStatus::Completed))
            .count();
        
        let features_total = self.progress_tracker.progress.features.len();
        
        Ok(format!(
            "Session: {}\nFeatures completed: {}/{} ({:.1}%)\nContext used: {}/{} tokens\nGit commits: {}",
            session.agent_type,
            features_completed,
            features_total,
            (features_completed as f32 / features_total as f32) * 100.0,
            self.context_used,
            self.context_budget,
            session.git_commits.len()
        ))
    }
}
