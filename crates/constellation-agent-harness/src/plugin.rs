//! Plugin system for language/framework adapters
//!
//! Provides a plugin architecture that allows the agent harness to work with
//! any programming language, framework, or toolchain through adapters.

use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

use crate::adapter::{FrameworkAdapter, LanguageAdapter, TestingAdapter};
use crate::error::{Error, Result};

/// Plugin trait for extensible functionality
pub trait Plugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;

    /// Plugin version
    fn version(&self) -> &str;

    /// Plugin description
    fn description(&self) -> &str;

    /// Initialize plugin with configuration
    fn initialize(&mut self, config: &PluginConfig) -> Result<()>;

    /// Shutdown plugin
    fn shutdown(&self) -> Result<()>;

    /// Get plugin as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get plugin as mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin name
    pub name: String,

    /// Plugin version
    pub version: String,

    /// Configuration options
    pub options: HashMap<String, serde_json::Value>,

    /// Plugin dependencies
    pub dependencies: Vec<String>,

    /// Plugin priority (higher = loaded earlier)
    pub priority: i32,
}

/// Plugin registry for managing plugins
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
    configs: HashMap<String, PluginConfig>,
    adapters: PluginAdapters,
}

/// Available adapters in the plugin system
#[derive(Default)]
struct PluginAdapters {
    language_adapters: HashMap<String, Arc<dyn LanguageAdapter>>,
    framework_adapters: HashMap<String, Arc<dyn FrameworkAdapter>>,
    testing_adapters: HashMap<String, Arc<dyn TestingAdapter>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            configs: HashMap::new(),
            adapters: PluginAdapters::default(),
        }
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, config: PluginConfig, plugin: Box<dyn Plugin>) -> Result<()> {
        let name = config.name.clone();

        // Initialize plugin
        let mut plugin = plugin;
        plugin.initialize(&config)?;

        // Store plugin and config
        self.plugins.insert(name.clone(), plugin);
        self.configs.insert(name, config);

        Ok(())
    }

    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// Get a mutable plugin by name
    pub fn get_plugin_mut(&mut self, name: &str) -> Option<&mut dyn Plugin> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            Some(plugin.as_mut())
        } else {
            None
        }
    }

    /// Get plugin configuration
    pub fn get_config(&self, name: &str) -> Option<&PluginConfig> {
        self.configs.get(name)
    }

    /// Register a language adapter
    pub fn register_language_adapter(&mut self, name: &str, adapter: Arc<dyn LanguageAdapter>) {
        self.adapters
            .language_adapters
            .insert(name.to_string(), adapter);
    }

    /// Register a framework adapter
    pub fn register_framework_adapter(&mut self, name: &str, adapter: Arc<dyn FrameworkAdapter>) {
        self.adapters
            .framework_adapters
            .insert(name.to_string(), adapter);
    }

    /// Register a testing adapter
    pub fn register_testing_adapter(&mut self, name: &str, adapter: Arc<dyn TestingAdapter>) {
        self.adapters
            .testing_adapters
            .insert(name.to_string(), adapter);
    }

    /// Detect and load appropriate adapters for a project
    pub fn detect_adapters(&self, project_path: &Path) -> Result<DetectedAdapters> {
        let mut detected = DetectedAdapters::default();

        // Detect language
        for (name, adapter) in &self.adapters.language_adapters {
            if adapter.detect(project_path)? {
                detected.language = Some(name.clone());
                break;
            }
        }

        // Detect framework
        for (name, adapter) in &self.adapters.framework_adapters {
            if adapter.detect(project_path)? {
                detected.framework = Some(name.clone());
                break;
            }
        }

        // Detect testing framework
        for (name, adapter) in &self.adapters.testing_adapters {
            if adapter.detect(project_path)? {
                detected.testing = Some(name.clone());
                break;
            }
        }

        Ok(detected)
    }

    /// Get language adapter
    pub fn get_language_adapter(&self, name: &str) -> Option<Arc<dyn LanguageAdapter>> {
        self.adapters.language_adapters.get(name).cloned()
    }

    /// Get framework adapter
    pub fn get_framework_adapter(&self, name: &str) -> Option<Arc<dyn FrameworkAdapter>> {
        self.adapters.framework_adapters.get(name).cloned()
    }

    /// Get testing adapter
    pub fn get_testing_adapter(&self, name: &str) -> Option<Arc<dyn TestingAdapter>> {
        self.adapters.testing_adapters.get(name).cloned()
    }

    /// Load plugins from a directory
    pub fn load_from_directory(&mut self, directory: &Path) -> Result<()> {
        // This would typically use dynamic loading (libloading)
        // For now, we'll implement a placeholder

        // In a real implementation, this would:
        // 1. Scan directory for plugin files (.so, .dylib, .dll)
        // 2. Load each plugin dynamically
        // 3. Register them with the registry

        // For now, we'll just log that we would load plugins
        log::info!("Would load plugins from: {}", directory.display());

        Ok(())
    }

    /// Shutdown all plugins
    pub fn shutdown(&mut self) -> Result<()> {
        let mut errors = Vec::new();

        for (name, plugin) in &mut self.plugins {
            if let Err(e) = plugin.shutdown() {
                errors.push(format!("Failed to shutdown plugin {}: {}", name, e));
            }
        }

        if !errors.is_empty() {
            return Err(Error::PluginError(errors.join(", ")));
        }

        Ok(())
    }
}

/// Detected adapters for a project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetectedAdapters {
    /// Detected programming language
    pub language: Option<String>,

    /// Detected framework
    pub framework: Option<String>,

    /// Detected testing framework
    pub testing: Option<String>,
}

impl DetectedAdapters {
    /// Check if all necessary adapters are detected
    pub fn is_complete(&self) -> bool {
        self.language.is_some() && self.testing.is_some()
    }

    /// Get missing adapter types
    pub fn missing(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();

        if self.language.is_none() {
            missing.push("language");
        }

        if self.testing.is_none() {
            missing.push("testing");
        }

        missing
    }
}

/// Built-in plugins
pub mod builtin {
    use super::*;

    /// Git integration plugin
    pub struct GitPlugin {
        name: String,
        version: String,
        description: String,
        initialized: bool,
    }

    impl Default for GitPlugin {
        fn default() -> Self {
            Self::new()
        }
    }

    impl GitPlugin {
        pub fn new() -> Self {
            Self {
                name: "git".to_string(),
                version: "1.0.0".to_string(),
                description: "Git version control integration".to_string(),
                initialized: false,
            }
        }
    }

    impl Plugin for GitPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            &self.version
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn initialize(&mut self, _config: &PluginConfig) -> Result<()> {
            self.initialized = true;
            log::info!("Git plugin initialized");
            Ok(())
        }

        fn shutdown(&self) -> Result<()> {
            log::info!("Git plugin shutdown");
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    /// Memory integration plugin
    pub struct MemoryPlugin {
        name: String,
        version: String,
        description: String,
        initialized: bool,
    }

    impl Default for MemoryPlugin {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MemoryPlugin {
        pub fn new() -> Self {
            Self {
                name: "memory".to_string(),
                version: "1.0.0".to_string(),
                description: "Neuroscience-inspired memory system integration".to_string(),
                initialized: false,
            }
        }
    }

    impl Plugin for MemoryPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            &self.version
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn initialize(&mut self, _config: &PluginConfig) -> Result<()> {
            self.initialized = true;
            log::info!("Memory plugin initialized");
            Ok(())
        }

        fn shutdown(&self) -> Result<()> {
            log::info!("Memory plugin shutdown");
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }
}

/// Plugin-related errors
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Plugin dependency missing: {0}")]
    DependencyMissing(String),

    #[error("Plugin configuration error: {0}")]
    ConfigError(String),
}
