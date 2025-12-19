//! Skill system with progressive disclosure
//!
//! Implements a skill-based system where agents can learn and execute skills
//! with progressive disclosure of complexity. Skills are stored in SKILL.md
//! format and can be discovered, loaded, and executed dynamically.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::plugin::PluginRegistry;

/// Skill definition in SKILL.md format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Skill metadata
    pub metadata: SkillMetadata,

    /// Skill content with progressive disclosure
    pub content: SkillContent,

    /// Execution history
    pub history: Vec<SkillExecution>,

    /// Dependencies on other skills
    pub dependencies: Vec<SkillId>,

    /// Prerequisites for learning this skill
    pub prerequisites: Vec<Prerequisite>,
}

impl Skill {
    /// Create a new skill
    pub fn new(name: String, difficulty: DifficultyLevel) -> Self {
        let id = name.to_lowercase().replace(" ", "_");
        Self {
            metadata: SkillMetadata {
                id: id.clone(),
                name,
                description: String::new(),
                version: "1.0.0".to_string(),
                author: "system".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                tags: HashSet::new(),
                category: SkillCategory::Architecture,
                difficulty,
                estimated_learning_time: 60, // 1 hour default
                success_rate: 0.0,
                usage_count: 0,
            },
            content: SkillContent {
                overview: String::new(),
                basic: None,
                advanced: None,
                expert: None,
                examples: Vec::new(),
                pitfalls: Vec::new(),
                related_skills: Vec::new(),
                templates: HashMap::new(),
                configuration: HashMap::new(),
            },
            history: Vec::new(),
            dependencies: Vec::new(),
            prerequisites: Vec::new(),
        }
    }
}

/// Skill metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Unique skill identifier
    pub id: SkillId,

    /// Skill name
    pub name: String,

    /// Skill description
    pub description: String,

    /// Skill version
    pub version: String,

    /// Author of the skill
    pub author: String,

    /// Creation date
    pub created_at: DateTime<Utc>,

    /// Last updated date
    pub updated_at: DateTime<Utc>,

    /// Tags for categorization
    pub tags: HashSet<String>,

    /// Skill category
    pub category: SkillCategory,

    /// Skill difficulty level
    pub difficulty: DifficultyLevel,

    /// Estimated time to learn (in minutes)
    pub estimated_learning_time: u32,

    /// Success rate (0.0 to 1.0)
    pub success_rate: f32,

    /// Usage count
    pub usage_count: u32,
}

/// Skill content with progressive disclosure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContent {
    /// Overview (visible to all)
    pub overview: String,

    /// Basic instructions (visible after prerequisites met)
    pub basic: Option<String>,

    /// Advanced techniques (visible after basic mastery)
    pub advanced: Option<String>,

    /// Expert insights (visible after advanced mastery)
    pub expert: Option<String>,

    /// Examples and use cases
    pub examples: Vec<Example>,

    /// Common pitfalls and how to avoid them
    pub pitfalls: Vec<Pitfall>,

    /// Related skills and when to use them
    pub related_skills: Vec<RelatedSkill>,

    /// Templates and code snippets
    pub templates: HashMap<String, String>,

    /// Configuration options
    pub configuration: HashMap<String, serde_json::Value>,
}

/// Skill execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecution {
    /// Execution timestamp
    pub timestamp: DateTime<Utc>,

    /// Agent that executed the skill
    pub agent_id: String,

    /// Execution context
    pub context: ExecutionContext,

    /// Execution result
    pub result: ExecutionResult,

    /// Execution duration in milliseconds
    pub duration_ms: u64,

    /// Feedback and lessons learned
    pub feedback: Option<String>,
}

/// Execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Project path
    pub project_path: PathBuf,

    /// Language/framework context
    pub language: String,
    pub framework: Option<String>,

    /// Task description
    pub task: String,

    /// Input parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Environment variables
    pub environment: HashMap<String, String>,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Success status
    pub success: bool,

    /// Output produced
    pub output: String,

    /// Errors encountered
    pub errors: Vec<String>,

    /// Artifacts created
    pub artifacts: Vec<PathBuf>,

    /// Metrics collected
    pub metrics: HashMap<String, f32>,
}

/// Prerequisite for learning a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prerequisite {
    /// Prerequisite type
    pub prerequisite_type: PrerequisiteType,

    /// Required value
    pub value: String,

    /// Minimum required level
    pub min_level: f32,
}

/// Example use case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    /// Example description
    pub description: String,

    /// Example code or command
    pub code: String,

    /// Expected output
    pub expected_output: String,

    /// Explanation
    pub explanation: String,
}

/// Common pitfall
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pitfall {
    /// Pitfall description
    pub description: String,

    /// How to recognize it
    pub recognition: String,

    /// How to avoid it
    pub avoidance: String,

    /// How to fix it if it occurs
    pub fix: String,
}

/// Related skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedSkill {
    /// Related skill ID
    pub skill_id: SkillId,

    /// Relationship type
    pub relationship: RelationshipType,

    /// When to use this related skill
    pub when_to_use: String,
}

/// Skill identifier
pub type SkillId = String;

/// Skill category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillCategory {
    /// Language-specific skills
    Language,

    /// Framework-specific skills
    Framework,

    /// Testing skills
    Testing,

    /// Deployment skills
    Deployment,

    /// Debugging skills
    Debugging,

    /// Optimization skills
    Optimization,

    /// Architecture skills
    Architecture,

    /// Tool usage skills
    Tool,

    /// Process skills
    Process,

    /// Communication skills
    Communication,
}

/// Difficulty level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Prerequisite type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrerequisiteType {
    /// Another skill that must be mastered
    Skill,

    /// Minimum experience level
    Experience,

    /// Specific tool knowledge
    Tool,

    /// Language proficiency
    Language,

    /// Framework knowledge
    Framework,
}

/// Relationship type between skills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Alternative approach
    Alternative,

    /// Complementary skill
    Complementary,

    /// Prerequisite skill
    Prerequisite,

    /// Follow-up skill
    FollowUp,

    /// Specialized version
    Specialization,
}

/// Skill executor
pub struct SkillExecutor {
    /// Skill registry
    registry: SkillRegistry,

    /// Plugin registry for adapter access
    plugin_registry: Arc<PluginRegistry>,

    /// Execution history
    history: Vec<SkillExecution>,

    /// Agent skill levels
    skill_levels: HashMap<SkillId, f32>,
}

/// Skill registry for managing skills
pub struct SkillRegistry {
    skills: HashMap<SkillId, Skill>,
    by_category: HashMap<SkillCategory, HashSet<SkillId>>,
    by_tag: HashMap<String, HashSet<SkillId>>,
    by_difficulty: HashMap<DifficultyLevel, HashSet<SkillId>>,
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillRegistry {
    /// Create a new skill registry
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            by_category: HashMap::new(),
            by_tag: HashMap::new(),
            by_difficulty: HashMap::new(),
        }
    }

    /// Register a skill
    pub fn register_skill(&mut self, skill: Skill) -> Result<()> {
        let id = skill.metadata.id.clone();

        // Update indices
        self.by_category
            .entry(skill.metadata.category)
            .or_default()
            .insert(id.clone());

        for tag in &skill.metadata.tags {
            self.by_tag
                .entry(tag.clone())
                .or_default()
                .insert(id.clone());
        }

        self.by_difficulty
            .entry(skill.metadata.difficulty)
            .or_default()
            .insert(id.clone());

        // Store skill
        self.skills.insert(id, skill);

        Ok(())
    }

    /// Load skills from a directory
    pub fn load_from_directory(&mut self, directory: &Path) -> Result<()> {
        if !directory.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Ok(skill) = self.load_skill_from_file(&path) {
                    self.register_skill(skill)?;
                }
            }
        }

        Ok(())
    }

    /// Load a skill from a SKILL.md file
    fn load_skill_from_file(&self, path: &Path) -> Result<Skill> {
        let content = std::fs::read_to_string(path)?;
        self.parse_skill_markdown(&content, path)
    }

    /// Parse SKILL.md format
    fn parse_skill_markdown(&self, _content: &str, path: &Path) -> Result<Skill> {
        // Simplified implementation - create a basic skill
        // Note: In a real implementation, this would parse YAML frontmatter
        let skill_name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Create a basic skill
        let skill = Skill::new(skill_name, DifficultyLevel::Intermediate);

        Ok(skill)
    }

    /// Parse content sections from markdown
    #[allow(dead_code)]
    fn parse_content_sections(&self, _content: &str) -> SkillContent {
        // Simplified implementation - return empty content
        SkillContent {
            overview: String::new(),
            basic: None,
            advanced: None,
            expert: None,
            examples: Vec::new(),
            pitfalls: Vec::new(),
            related_skills: Vec::new(),
            templates: HashMap::new(),
            configuration: HashMap::new(),
        }
    }

    /// Get skill by ID
    pub fn get_skill(&self, id: &SkillId) -> Option<&Skill> {
        self.skills.get(id)
    }

    /// Find skills by category
    pub fn find_by_category(&self, category: SkillCategory) -> Vec<&Skill> {
        self.by_category
            .get(&category)
            .map(|ids| ids.iter().filter_map(|id| self.skills.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find skills by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<&Skill> {
        self.by_tag
            .get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.skills.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find skills by difficulty
    pub fn find_by_difficulty(&self, difficulty: DifficultyLevel) -> Vec<&Skill> {
        self.by_difficulty
            .get(&difficulty)
            .map(|ids| ids.iter().filter_map(|id| self.skills.get(id)).collect())
            .unwrap_or_default()
    }

    /// Search skills by query
    pub fn search(&self, query: &str) -> Vec<&Skill> {
        let query_lower = query.to_lowercase();

        self.skills
            .values()
            .filter(|skill| {
                skill.metadata.name.to_lowercase().contains(&query_lower)
                    || skill
                        .metadata
                        .description
                        .to_lowercase()
                        .contains(&query_lower)
                    || skill
                        .metadata
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }
}

impl SkillExecutor {
    /// Create a new skill executor
    pub fn new(registry: SkillRegistry, plugin_registry: Arc<PluginRegistry>) -> Self {
        Self {
            registry,
            plugin_registry,
            history: Vec::new(),
            skill_levels: HashMap::new(),
        }
    }

    /// Execute a skill
    pub fn execute_skill(
        &mut self,
        skill_id: &SkillId,
        context: &ExecutionContext,
        agent_id: &str,
    ) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();

        // Get skill
        let skill = self
            .registry
            .get_skill(skill_id)
            .ok_or_else(|| Error::SkillError(format!("Skill not found: {}", skill_id)))?;

        // Check prerequisites
        self.check_prerequisites(skill)?;

        // Get appropriate content based on skill level
        let content = self.get_skill_content(skill, agent_id);

        // Execute skill based on category
        let result = match skill.metadata.category {
            SkillCategory::Language => self.execute_language_skill(skill, &content, context),
            SkillCategory::Framework => self.execute_framework_skill(skill, &content, context),
            SkillCategory::Testing => self.execute_testing_skill(skill, &content, context),
            SkillCategory::Deployment => self.execute_deployment_skill(skill, &content, context),
            SkillCategory::Debugging => self.execute_debugging_skill(skill, &content, context),
            SkillCategory::Optimization => {
                self.execute_optimization_skill(skill, &content, context)
            }
            SkillCategory::Architecture => {
                self.execute_architecture_skill(skill, &content, context)
            }
            SkillCategory::Tool => self.execute_tool_skill(skill, &content, context),
            SkillCategory::Process => self.execute_process_skill(skill, &content, context),
            SkillCategory::Communication => {
                self.execute_communication_skill(skill, &content, context)
            }
        }?;

        let duration = start_time.elapsed();

        // Record execution
        let execution = SkillExecution {
            timestamp: Utc::now(),
            agent_id: agent_id.to_string(),
            context: context.clone(),
            result: result.clone(),
            duration_ms: duration.as_millis() as u64,
            feedback: None,
        };

        self.history.push(execution);

        // Update skill level
        self.update_skill_level(skill_id, agent_id, &result);

        // Update skill success rate
        // (In a real implementation, this would update the skill in the registry)

        Ok(result)
    }

    /// Get skill content appropriate for agent's skill level
    fn get_skill_content(&self, skill: &Skill, _agent_id: &str) -> String {
        let skill_level = self
            .skill_levels
            .get(&skill.metadata.id)
            .copied()
            .unwrap_or(0.0);

        let mut content = String::new();

        // Always include overview
        content.push_str(&skill.content.overview);
        content.push_str("\n\n");

        // Include basic if skill level > 0.3
        if skill_level > 0.3 {
            if let Some(basic) = &skill.content.basic {
                content.push_str("## Basic Instructions\n");
                content.push_str(basic);
                content.push_str("\n\n");
            }
        }

        // Include advanced if skill level > 0.6
        if skill_level > 0.6 {
            if let Some(advanced) = &skill.content.advanced {
                content.push_str("## Advanced Techniques\n");
                content.push_str(advanced);
                content.push_str("\n\n");
            }
        }

        // Include expert if skill level > 0.9
        if skill_level > 0.9 {
            if let Some(expert) = &skill.content.expert {
                content.push_str("## Expert Insights\n");
                content.push_str(expert);
                content.push_str("\n\n");
            }
        }

        // Include examples
        if !skill.content.examples.is_empty() {
            content.push_str("## Examples\n");
            for example in &skill.content.examples {
                content.push_str(&format!("### {}\n", example.description));
                content.push_str(&format!("```\n{}\n```\n", example.code));
                content.push_str(&format!("**Expected:** {}\n", example.expected_output));
                content.push_str(&format!("**Explanation:** {}\n\n", example.explanation));
            }
        }

        content
    }

    /// Check if agent meets skill prerequisites
    fn check_prerequisites(&self, skill: &Skill) -> Result<()> {
        for prerequisite in &skill.prerequisites {
            if prerequisite.prerequisite_type == PrerequisiteType::Skill {
                let skill_level = self
                    .skill_levels
                    .get(&prerequisite.value)
                    .copied()
                    .unwrap_or(0.0);

                if skill_level < prerequisite.min_level {
                    return Err(Error::SkillError(format!(
                        "Prerequisite skill '{}' not met. Required level: {}, Current level: {}",
                        prerequisite.value, prerequisite.min_level, skill_level
                    )));
                }
            }
            // Other prerequisite types would be checked here
        }

        Ok(())
    }

    /// Update agent's skill level based on execution result
    fn update_skill_level(
        &mut self,
        skill_id: &SkillId,
        _agent_id: &str,
        result: &ExecutionResult,
    ) {
        let current_level = self.skill_levels.get(skill_id).copied().unwrap_or(0.0);

        let new_level = if result.success {
            // Successful execution increases skill level
            let increase = 0.1 * (1.0 - current_level); // Diminishing returns
            current_level + increase
        } else {
            // Failed execution decreases skill level slightly
            current_level * 0.95
        };

        self.skill_levels
            .insert(skill_id.clone(), new_level.min(1.0));
    }

    // Skill execution methods for different categories

    fn execute_language_skill(
        &self,
        skill: &Skill,
        content: &str,
        context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        // Use language adapter from plugin registry
        if let Some(_adapter) = self.plugin_registry.get_language_adapter(&context.language) {
            // Parse skill content for specific actions
            // This would be more sophisticated in a real implementation

            let output = format!(
                "Executing language skill: {}\n\n{}",
                skill.metadata.name, content
            );

            Ok(ExecutionResult {
                success: true,
                output,
                errors: Vec::new(),
                artifacts: Vec::new(),
                metrics: HashMap::new(),
            })
        } else {
            Err(Error::SkillError(format!(
                "No language adapter found for: {}",
                context.language
            )))
        }
    }

    fn execute_framework_skill(
        &self,
        skill: &Skill,
        content: &str,
        _context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        // Similar to language skill but with framework adapter
        let output = format!(
            "Executing framework skill: {}\n\n{}",
            skill.metadata.name, content
        );

        Ok(ExecutionResult {
            success: true,
            output,
            errors: Vec::new(),
            artifacts: Vec::new(),
            metrics: HashMap::new(),
        })
    }

    fn execute_testing_skill(
        &self,
        skill: &Skill,
        content: &str,
        _context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        // Use testing adapter
        let output = format!(
            "Executing testing skill: {}\n\n{}",
            skill.metadata.name, content
        );

        Ok(ExecutionResult {
            success: true,
            output,
            errors: Vec::new(),
            artifacts: Vec::new(),
            metrics: HashMap::new(),
        })
    }

    // Other skill execution methods would be similar

    fn execute_deployment_skill(
        &self,
        skill: &Skill,
        content: &str,
        context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        self.generic_skill_execution(skill, content, context)
    }

    fn execute_debugging_skill(
        &self,
        skill: &Skill,
        content: &str,
        context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        self.generic_skill_execution(skill, content, context)
    }

    fn execute_optimization_skill(
        &self,
        skill: &Skill,
        content: &str,
        context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        self.generic_skill_execution(skill, content, context)
    }

    fn execute_architecture_skill(
        &self,
        skill: &Skill,
        content: &str,
        context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        self.generic_skill_execution(skill, content, context)
    }

    fn execute_tool_skill(
        &self,
        skill: &Skill,
        content: &str,
        context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        self.generic_skill_execution(skill, content, context)
    }

    fn execute_process_skill(
        &self,
        skill: &Skill,
        content: &str,
        context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        self.generic_skill_execution(skill, content, context)
    }

    fn execute_communication_skill(
        &self,
        skill: &Skill,
        content: &str,
        context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        self.generic_skill_execution(skill, content, context)
    }

    fn generic_skill_execution(
        &self,
        skill: &Skill,
        content: &str,
        _context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        let output = format!("Executing skill: {}\n\n{}", skill.metadata.name, content);

        Ok(ExecutionResult {
            success: true,
            output,
            errors: Vec::new(),
            artifacts: Vec::new(),
            metrics: HashMap::new(),
        })
    }

    /// Get agent's skill level
    pub fn get_skill_level(&self, skill_id: &SkillId) -> f32 {
        self.skill_levels.get(skill_id).copied().unwrap_or(0.0)
    }

    /// Get execution history
    pub fn get_history(&self) -> &[SkillExecution] {
        &self.history
    }

    /// Get recommended skills for agent based on current skill levels
    pub fn get_recommended_skills(&self, _agent_id: &str) -> Vec<&Skill> {
        let mut recommendations = Vec::new();

        for skill in self.registry.skills.values() {
            // Check if agent already has high level in this skill
            let current_level = self.get_skill_level(&skill.metadata.id);
            if current_level > 0.8 {
                continue; // Already proficient
            }

            // Check prerequisites
            let mut prerequisites_met = true;
            for prerequisite in &skill.prerequisites {
                if prerequisite.prerequisite_type == PrerequisiteType::Skill {
                    let prereq_level = self.get_skill_level(&prerequisite.value);
                    if prereq_level < prerequisite.min_level {
                        prerequisites_met = false;
                        break;
                    }
                }
            }

            if prerequisites_met {
                recommendations.push(skill);
            }
        }

        // Sort by difficulty and estimated learning time
        recommendations.sort_by(|a, b| {
            a.metadata.difficulty.cmp(&b.metadata.difficulty).then(
                a.metadata
                    .estimated_learning_time
                    .cmp(&b.metadata.estimated_learning_time),
            )
        });

        recommendations
    }
}
