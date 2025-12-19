//! Adapter system for language/framework/toolchain independence
//!
//! Provides abstract interfaces for working with different programming languages,
//! frameworks, and testing tools without hard-coding specific implementations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};

/// Language adapter for programming language-specific operations
pub trait LanguageAdapter: Send + Sync {
    /// Adapter name
    fn name(&self) -> &str;

    /// Detect if this language is used in the project
    fn detect(&self, project_path: &Path) -> Result<bool>;

    /// Initialize a new project in this language
    fn initialize_project(&self, project_path: &Path, project_name: &str) -> Result<()>;

    /// Create a new module/file in the project
    fn create_module(&self, project_path: &Path, module_name: &str) -> Result<PathBuf>;

    /// Compile/build the project
    fn compile(&self, project_path: &Path) -> Result<CompilationResult>;

    /// Get language-specific file extensions
    fn file_extensions(&self) -> &[&str];

    /// Get language-specific template for a given type
    fn get_template(&self, template_type: &str) -> Result<String>;

    /// Analyze code for patterns and issues
    fn analyze_code(&self, code: &str) -> Result<CodeAnalysis>;
}

/// Framework adapter for framework-specific operations
pub trait FrameworkAdapter: Send + Sync {
    /// Adapter name
    fn name(&self) -> &str;

    /// Detect if this framework is used in the project
    fn detect(&self, project_path: &Path) -> Result<bool>;

    /// Initialize framework in project
    fn initialize(&self, project_path: &Path) -> Result<()>;

    /// Create a new component in the framework
    fn create_component(&self, project_path: &Path, component_name: &str) -> Result<PathBuf>;

    /// Run framework-specific development server
    fn run_dev_server(&self, project_path: &Path) -> Result<()>;

    /// Build for production
    fn build_production(&self, project_path: &Path) -> Result<BuildResult>;

    /// Get framework-specific configuration
    fn get_config(&self) -> Result<FrameworkConfig>;
}

/// Testing adapter for test framework integration
pub trait TestingAdapter: Send + Sync {
    /// Adapter name
    fn name(&self) -> &str;

    /// Detect if this testing framework is used
    fn detect(&self, project_path: &Path) -> Result<bool>;

    /// Initialize testing in project
    fn initialize(&self, project_path: &Path) -> Result<()>;

    /// Run tests
    fn run_tests(&self, project_path: &Path, test_filter: Option<&str>) -> Result<TestResults>;

    /// Run specific test file
    fn run_test_file(&self, project_path: &Path, test_file: &Path) -> Result<TestResults>;

    /// Generate test coverage report
    fn generate_coverage(&self, project_path: &Path) -> Result<CoverageReport>;

    /// Create a new test
    fn create_test(&self, project_path: &Path, test_name: &str) -> Result<PathBuf>;
}

/// Compilation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationResult {
    pub success: bool,
    pub output: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub duration_ms: u64,
    pub artifacts: Vec<PathBuf>,
}

/// Code analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysis {
    pub complexity: f32,
    pub maintainability: f32,
    pub security_issues: Vec<SecurityIssue>,
    pub performance_issues: Vec<PerformanceIssue>,
    pub style_violations: Vec<StyleViolation>,
    pub suggestions: Vec<Suggestion>,
}

/// Security issue in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub severity: Severity,
    pub description: String,
    pub location: CodeLocation,
    pub recommendation: String,
}

/// Performance issue in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIssue {
    pub impact: ImpactLevel,
    pub description: String,
    pub location: CodeLocation,
    pub optimization: String,
}

/// Style violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleViolation {
    pub rule: String,
    pub description: String,
    pub location: CodeLocation,
    pub fix: Option<String>,
}

/// Code suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub category: SuggestionCategory,
    pub description: String,
    pub location: CodeLocation,
    pub code: String,
}

/// Code location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}

/// Severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Impact levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
}

/// Suggestion categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Refactoring,
    Optimization,
    Documentation,
    Testing,
    Security,
    Style,
}

/// Build result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub output: String,
    pub artifacts: Vec<PathBuf>,
    pub size_bytes: u64,
    pub duration_ms: u64,
}

/// Test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: u64,
    pub failures: Vec<TestFailure>,
    pub coverage: Option<f32>,
}

/// Test failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    pub test_name: String,
    pub message: String,
    pub location: CodeLocation,
    pub stack_trace: Option<String>,
}

/// Coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub total_lines: usize,
    pub covered_lines: usize,
    pub coverage_percentage: f32,
    pub uncovered_lines: Vec<UncoveredLine>,
    pub by_file: HashMap<PathBuf, FileCoverage>,
}

/// Uncovered line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredLine {
    pub file: PathBuf,
    pub line: usize,
    pub code: String,
}

/// File coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    pub total_lines: usize,
    pub covered_lines: usize,
    pub coverage_percentage: f32,
}

/// Framework configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkConfig {
    pub name: String,
    pub version: String,
    pub dependencies: HashMap<String, String>,
    pub scripts: HashMap<String, String>,
    pub config_files: Vec<PathBuf>,
}

/// Built-in adapters
pub mod builtin {
    use super::*;

    /// Rust language adapter
    pub struct RustAdapter;

    impl Default for RustAdapter {
        fn default() -> Self {
            Self::new()
        }
    }

    impl RustAdapter {
        pub fn new() -> Self {
            Self
        }
    }

    impl LanguageAdapter for RustAdapter {
        fn name(&self) -> &str {
            "rust"
        }

        fn detect(&self, project_path: &Path) -> Result<bool> {
            Ok(project_path.join("Cargo.toml").exists())
        }

        fn initialize_project(&self, project_path: &Path, project_name: &str) -> Result<()> {
            let output = Command::new("cargo")
                .arg("new")
                .arg(project_name)
                .current_dir(project_path.parent().unwrap_or(project_path))
                .output()?;

            if !output.status.success() {
                return Err(Error::AdapterError(format!(
                    "Failed to initialize Rust project: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }

            Ok(())
        }

        fn create_module(&self, project_path: &Path, module_name: &str) -> Result<PathBuf> {
            let module_path = project_path.join("src").join(format!("{}.rs", module_name));

            let template = r#"// Module: {module_name}

pub struct {module_name} {{
    // TODO: Add fields
}}

impl {module_name} {{
    pub fn new() -> Self {{
        Self {{
            // TODO: Initialize fields
        }}
    }}
    
    // TODO: Add methods
}}

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn test_{module_name}_creation() {{
        let instance = {module_name}::new();
        assert!(true); // TODO: Add actual assertions
    }}
}}"#;

            let content = template.replace("{module_name}", module_name);
            std::fs::write(&module_path, content)?;

            Ok(module_path)
        }

        fn compile(&self, project_path: &Path) -> Result<CompilationResult> {
            let start_time = std::time::Instant::now();

            let output = Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(project_path)
                .output()?;

            let duration = start_time.elapsed();

            let output_str = String::from_utf8_lossy(&output.stdout).to_string();
            let error_str = String::from_utf8_lossy(&output.stderr).to_string();

            // Simple parsing of warnings and errors
            let warnings: Vec<String> = output_str
                .lines()
                .filter(|line| line.contains("warning:"))
                .map(|s| s.to_string())
                .collect();

            let errors: Vec<String> = error_str
                .lines()
                .filter(|line| line.contains("error:"))
                .map(|s| s.to_string())
                .collect();

            // Find artifacts
            let artifacts = if output.status.success() {
                let target_dir = project_path.join("target").join("release");
                let mut artifacts = Vec::new();

                if let Ok(entries) = std::fs::read_dir(&target_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            artifacts.push(path);
                        }
                    }
                }

                artifacts
            } else {
                Vec::new()
            };

            Ok(CompilationResult {
                success: output.status.success(),
                output: output_str,
                warnings,
                errors,
                duration_ms: duration.as_millis() as u64,
                artifacts,
            })
        }

        fn file_extensions(&self) -> &[&str] {
            &["rs", "toml"]
        }

        fn get_template(&self, template_type: &str) -> Result<String> {
            match template_type {
                "struct" => Ok(r#"pub struct {name} {{
    // Fields
}}

impl {name} {{
    pub fn new() -> Self {{
        Self {{
            // Initialize fields
        }}
    }}
}}"#
                .to_string()),
                "enum" => Ok(r#"pub enum {name} {{
    // Variants
}}"#
                .to_string()),
                "trait" => Ok(r#"pub trait {name} {{
    // Method signatures
}}"#
                .to_string()),
                "function" => Ok(
                    r#"pub fn {name}() -> Result<(), Box<dyn std::error::Error>> {{
    // Implementation
    Ok(())
}}"#
                    .to_string(),
                ),
                _ => Err(Error::AdapterError(format!(
                    "Unknown template type: {}",
                    template_type
                ))),
            }
        }

        fn analyze_code(&self, code: &str) -> Result<CodeAnalysis> {
            // Simple analysis for demonstration
            // In a real implementation, this would use rust-analyzer or similar

            let lines = code.lines().count();
            let complexity = (lines as f32 / 50.0).min(10.0); // Simple heuristic

            let mut security_issues = Vec::new();
            let mut performance_issues = Vec::new();
            let style_violations = Vec::new();
            let suggestions = Vec::new();

            // Check for common issues
            for (i, line) in code.lines().enumerate() {
                if line.contains("unwrap()") && !line.contains("// safe:") {
                    security_issues.push(SecurityIssue {
                        severity: Severity::Medium,
                        description: "Use of unwrap() without justification".to_string(),
                        location: CodeLocation {
                            file: PathBuf::from("unknown.rs"),
                            line: i + 1,
                            column: line.find("unwrap()").unwrap_or(0),
                        },
                        recommendation: "Consider using match or ? operator for error handling"
                            .to_string(),
                    });
                }

                if line.contains(".clone()") && line.matches(".clone()").count() > 1 {
                    performance_issues.push(PerformanceIssue {
                        impact: ImpactLevel::Medium,
                        description: "Multiple clones on same line".to_string(),
                        location: CodeLocation {
                            file: PathBuf::from("unknown.rs"),
                            line: i + 1,
                            column: line.find(".clone()").unwrap_or(0),
                        },
                        optimization: "Consider using references or Arc/Rc for shared ownership"
                            .to_string(),
                    });
                }
            }

            Ok(CodeAnalysis {
                complexity,
                maintainability: 1.0 - (complexity / 10.0),
                security_issues,
                performance_issues,
                style_violations,
                suggestions,
            })
        }
    }

    /// JavaScript/TypeScript adapter
    pub struct JavaScriptAdapter;

    impl Default for JavaScriptAdapter {
        fn default() -> Self {
            Self::new()
        }
    }

    impl JavaScriptAdapter {
        pub fn new() -> Self {
            Self
        }
    }

    impl LanguageAdapter for JavaScriptAdapter {
        fn name(&self) -> &str {
            "javascript"
        }

        fn detect(&self, project_path: &Path) -> Result<bool> {
            Ok(project_path.join("package.json").exists())
        }

        fn initialize_project(&self, project_path: &Path, project_name: &str) -> Result<()> {
            let package_json = project_path.join("package.json");

            if !package_json.exists() {
                let content = serde_json::json!({
                    "name": project_name,
                    "version": "1.0.0",
                    "description": "Project generated by Constellation",
                    "main": "index.js",
                    "scripts": {
                        "start": "node index.js",
                        "test": "jest"
                    },
                    "dependencies": {},
                    "devDependencies": {}
                });

                std::fs::write(package_json, serde_json::to_string_pretty(&content)?)?;
            }

            Ok(())
        }

        fn create_module(&self, project_path: &Path, module_name: &str) -> Result<PathBuf> {
            let module_path = project_path.join("src").join(format!("{}.js", module_name));

            let template = r#"// Module: {module_name}

class {module_name} {{
    constructor() {{
        // TODO: Initialize properties
    }}
    
    // TODO: Add methods
}}

module.exports = {module_name};"#;

            let content = template.replace("{module_name}", module_name);
            std::fs::write(&module_path, content)?;

            Ok(module_path)
        }

        fn compile(&self, _project_path: &Path) -> Result<CompilationResult> {
            // JavaScript doesn't have compilation in the traditional sense
            Ok(CompilationResult {
                success: true,
                output: "JavaScript is interpreted, no compilation needed".to_string(),
                warnings: Vec::new(),
                errors: Vec::new(),
                duration_ms: 0,
                artifacts: Vec::new(),
            })
        }

        fn file_extensions(&self) -> &[&str] {
            &["js", "ts", "jsx", "tsx", "json"]
        }

        fn get_template(&self, template_type: &str) -> Result<String> {
            match template_type {
                "class" => Ok(r#"class {name} {{
    constructor() {{
        // Initialize properties
    }}
    
    // Methods
}}"#
                .to_string()),
                "function" => Ok(r#"function {name}() {{
    // Implementation
}}"#
                .to_string()),
                "module" => Ok(r#"// Module: {name}

module.exports = {{
    // Exports
}};"#
                    .to_string()),
                _ => Err(Error::AdapterError(format!(
                    "Unknown template type: {}",
                    template_type
                ))),
            }
        }

        fn analyze_code(&self, code: &str) -> Result<CodeAnalysis> {
            // Simple analysis for JavaScript
            let lines = code.lines().count();
            let complexity = (lines as f32 / 30.0).min(10.0); // JavaScript tends to be more verbose

            Ok(CodeAnalysis {
                complexity,
                maintainability: 1.0 - (complexity / 10.0),
                security_issues: Vec::new(),
                performance_issues: Vec::new(),
                style_violations: Vec::new(),
                suggestions: Vec::new(),
            })
        }
    }

    /// Jest testing adapter
    pub struct JestAdapter;

    impl Default for JestAdapter {
        fn default() -> Self {
            Self::new()
        }
    }

    impl JestAdapter {
        pub fn new() -> Self {
            Self
        }
    }

    impl TestingAdapter for JestAdapter {
        fn name(&self) -> &str {
            "jest"
        }

        fn detect(&self, project_path: &Path) -> Result<bool> {
            let package_json_path = project_path.join("package.json");
            if package_json_path.exists() {
                let content = std::fs::read_to_string(package_json_path)?;
                let package_json: serde_json::Value = serde_json::from_str(&content)?;

                if let Some(deps) = package_json.get("devDependencies") {
                    if deps.get("jest").is_some() {
                        return Ok(true);
                    }
                }

                if let Some(scripts) = package_json.get("scripts") {
                    if let Some(test_script) = scripts.get("test") {
                        if test_script.as_str().unwrap_or("").contains("jest") {
                            return Ok(true);
                        }
                    }
                }
            }

            Ok(false)
        }

        fn initialize(&self, project_path: &Path) -> Result<()> {
            // Install jest if not already installed
            let output = Command::new("npm")
                .args(["install", "--save-dev", "jest", "@types/jest"])
                .current_dir(project_path)
                .output()?;

            if !output.status.success() {
                return Err(Error::AdapterError(format!(
                    "Failed to install Jest: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }

            // Create jest config
            let jest_config = project_path.join("jest.config.js");
            let config_content = r#"module.exports = {
    testEnvironment: 'node',
    testMatch: ['**/__tests__/**/*.js', '**/?(*.)+(spec|test).js'],
    collectCoverage: true,
    coverageDirectory: 'coverage',
};"#;

            std::fs::write(jest_config, config_content)?;

            Ok(())
        }

        fn run_tests(&self, project_path: &Path, test_filter: Option<&str>) -> Result<TestResults> {
            let mut command = Command::new("npx");
            command.arg("jest");

            if let Some(filter) = test_filter {
                command.arg("--testNamePattern").arg(filter);
            }

            command.arg("--coverage");
            command.current_dir(project_path);

            let output = command.output()?;
            let output_str = String::from_utf8_lossy(&output.stdout);

            // Parse jest output (simplified)
            let mut total_tests = 0;
            let mut passed = 0;
            let mut failed = 0;
            let skipped = 0;
            let mut coverage = None;

            for line in output_str.lines() {
                if line.contains("Tests:") {
                    // Parse: "Tests: 5 passed, 5 total"
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        total_tests = parts[3].parse().unwrap_or(0);
                        passed = parts[1].parse().unwrap_or(0);
                    }
                } else if line.contains("FAIL") {
                    failed += 1;
                } else if line.contains("All files") && line.contains("%") {
                    // Parse coverage: "All files | 100.00 | 100.00 | 100.00 | 100.00"
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 2 {
                        coverage = parts[1].trim().parse::<f32>().ok();
                    }
                }
            }

            Ok(TestResults {
                total_tests,
                passed,
                failed,
                skipped,
                duration_ms: 0, // Would parse from output
                failures: Vec::new(),
                coverage,
            })
        }

        fn run_test_file(&self, project_path: &Path, _test_file: &Path) -> Result<TestResults> {
            self.run_tests(project_path, None) // Simplified
        }

        fn generate_coverage(&self, project_path: &Path) -> Result<CoverageReport> {
            let test_results = self.run_tests(project_path, None)?;

            Ok(CoverageReport {
                total_lines: 0,
                covered_lines: 0,
                coverage_percentage: test_results.coverage.unwrap_or(0.0),
                uncovered_lines: Vec::new(),
                by_file: HashMap::new(),
            })
        }

        fn create_test(&self, project_path: &Path, test_name: &str) -> Result<PathBuf> {
            let test_dir = project_path.join("__tests__");
            if !test_dir.exists() {
                std::fs::create_dir_all(&test_dir)?;
            }

            let test_path = test_dir.join(format!("{}.test.js", test_name));

            let template = r#"describe('{test_name}', () => {{
    test('should work correctly', () => {{
        expect(true).toBe(true);
    }});
    
    // TODO: Add more tests
}});"#;

            let content = template.replace("{test_name}", test_name);
            std::fs::write(&test_path, content)?;

            Ok(test_path)
        }
    }
}
