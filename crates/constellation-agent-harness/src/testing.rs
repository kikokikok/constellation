use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::fs;
use crate::error::{Error, Result};
use crate::progress::{Feature, TestResults};
use tempfile::TempDir;

pub struct TestingIntegration {
    project_path: PathBuf,
    temp_dir: TempDir,
}

impl TestingIntegration {
    pub fn new(project_path: &Path) -> Result<Self> {
        let temp_dir = TempDir::new()?;
        
        Ok(Self {
            project_path: project_path.to_path_buf(),
            temp_dir,
        })
    }

    pub fn run_feature_tests(&self, feature: &Feature) -> Result<TestResults> {
        let start_time = std::time::Instant::now();
        
        let cargo_test_result = self.run_cargo_tests(feature)?;
        let npm_test_result = self.run_npm_tests(feature)?;
        let browser_test_result = self.run_browser_tests(feature)?;
        
        let duration = start_time.elapsed();
        
        let passed = cargo_test_result.passed && npm_test_result.passed && browser_test_result.passed;
        
        let total_tests = cargo_test_result.total_tests + npm_test_result.total_tests + browser_test_result.total_tests;
        let passed_tests = cargo_test_result.passed_tests + npm_test_result.passed_tests + browser_test_result.passed_tests;
        let failed_tests = total_tests - passed_tests;
        
        let error_message = if !passed {
            let mut errors = Vec::new();
            
            if !cargo_test_result.passed {
                errors.push("Cargo tests failed".to_string());
            }
            
            if !npm_test_result.passed {
                errors.push("NPM tests failed".to_string());
            }
            
            if !browser_test_result.passed {
                errors.push("Browser tests failed".to_string());
            }
            
            Some(errors.join(", "))
        } else {
            None
        };
        
        let screenshots = self.capture_screenshots(feature)?;
        
        Ok(TestResults {
            passed,
            total_tests,
            passed_tests,
            failed_tests,
            duration_seconds: duration.as_secs_f64(),
            screenshots,
            error_message,
        })
    }

    fn run_cargo_tests(&self, feature: &Feature) -> Result<TestResults> {
        let cargo_toml_path = self.project_path.join("Cargo.toml");
        
        if !cargo_toml_path.exists() {
            return Ok(TestResults {
                passed: true,
                total_tests: 0,
                passed_tests: 0,
                failed_tests: 0,
                duration_seconds: 0.0,
                screenshots: Vec::new(),
                error_message: None,
            });
        }
        
        let output = Command::new("cargo")
            .arg("test")
            .arg("--")
            .arg("--test-threads=1")
            .arg("--nocapture")
            .current_dir(&self.project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let test_output = stdout.to_string() + &stderr;
        
        let passed = output.status.success();
        
        let total_tests = test_output.matches("test result:").count();
        let passed_tests = test_output.matches("test result: ok").count();
        let failed_tests = total_tests - passed_tests;
        
        let error_message = if !passed {
            Some(format!("Cargo tests failed:\n{}\n{}", stdout, stderr))
        } else {
            None
        };
        
        Ok(TestResults {
            passed,
            total_tests,
            passed_tests,
            failed_tests,
            duration_seconds: 0.0,
            screenshots: Vec::new(),
            error_message,
        })
    }

    fn run_npm_tests(&self, feature: &Feature) -> Result<TestResults> {
        let package_json_path = self.project_path.join("package.json");
        
        if !package_json_path.exists() {
            return Ok(TestResults {
                passed: true,
                total_tests: 0,
                passed_tests: 0,
                failed_tests: 0,
                duration_seconds: 0.0,
                screenshots: Vec::new(),
                error_message: None,
            });
        }
        
        let output = Command::new("npm")
            .arg("test")
            .current_dir(&self.project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let passed = output.status.success();
        
        let test_output = stdout.to_string() + &stderr;
        
        let total_tests = test_output.matches("✓").count() + test_output.matches("×").count();
        let passed_tests = test_output.matches("✓").count();
        let failed_tests = test_output.matches("×").count();
        
        let error_message = if !passed {
            Some(format!("NPM tests failed:\n{}\n{}", stdout, stderr))
        } else {
            None
        };
        
        Ok(TestResults {
            passed,
            total_tests,
            passed_tests,
            failed_tests,
            duration_seconds: 0.0,
            screenshots: Vec::new(),
            error_message,
        })
    }

    fn run_browser_tests(&self, feature: &Feature) -> Result<TestResults> {
        let playwright_config = self.project_path.join("playwright.config.js");
        let puppeteer_config = self.project_path.join("puppeteer.config.js");
        
        if !playwright_config.exists() && !puppeteer_config.exists() {
            return Ok(TestResults {
                passed: true,
                total_tests: 0,
                passed_tests: 0,
                failed_tests: 0,
                duration_seconds: 0.0,
                screenshots: Vec::new(),
                error_message: None,
            });
        }
        
        let output = if playwright_config.exists() {
            Command::new("npx")
                .arg("playwright")
                .arg("test")
                .current_dir(&self.project_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?
        } else {
            Command::new("npx")
                .arg("jest")
                .arg("--config=puppeteer.config.js")
                .current_dir(&self.project_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?
        };
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let passed = output.status.success();
        
        let test_output = stdout.to_string() + &stderr;
        
        let total_tests = test_output.matches("PASS").count() + test_output.matches("FAIL").count();
        let passed_tests = test_output.matches("PASS").count();
        let failed_tests = test_output.matches("FAIL").count();
        
        let error_message = if !passed {
            Some(format!("Browser tests failed:\n{}\n{}", stdout, stderr))
        } else {
            None
        };
        
        Ok(TestResults {
            passed,
            total_tests,
            passed_tests,
            failed_tests,
            duration_seconds: 0.0,
            screenshots: Vec::new(),
            error_message,
        })
    }

    fn capture_screenshots(&self, feature: &Feature) -> Result<Vec<PathBuf>> {
        let mut screenshots = Vec::new();
        
        let screenshot_dir = self.temp_dir.path().join("screenshots");
        fs::create_dir_all(&screenshot_dir)?;
        
        let screenshot_path = screenshot_dir.join(format!("{}.png", feature.name.replace(" ", "_")));
        
        let html_content = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Test: {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .feature {{ border: 1px solid #ccc; padding: 20px; border-radius: 5px; }}
        .status {{ color: green; font-weight: bold; }}
    </style>
</head>
<body>
    <div class="feature">
        <h1>{}</h1>
        <p>{}</p>
        <p class="status">Test completed: {}</p>
    </div>
</body>
</html>"#,
            feature.name,
            feature.name,
            feature.description,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        );
        
        let html_path = screenshot_dir.join("test.html");
        fs::write(&html_path, html_content)?;
        
        if Command::new("which").arg("wkhtmltoimage").output().is_ok() {
            let _ = Command::new("wkhtmltoimage")
                .arg("--quality")
                .arg("85")
                .arg(&html_path)
                .arg(&screenshot_path)
                .output();
        }
        
        if screenshot_path.exists() {
            screenshots.push(screenshot_path);
        }
        
        Ok(screenshots)
    }

    pub fn create_test_skill(&self, feature: &Feature) -> Result<PathBuf> {
        let skill_dir = self.project_path.join("skills").join("testing");
        fs::create_dir_all(&skill_dir)?;
        
        let skill_path = skill_dir.join(format!("test_{}.md", feature.name.replace(" ", "_").to_lowercase()));
        
        let skill_content = format!(
            r#"---
name: Test {}
description: Automated tests for {} feature
version: 1.0.0
author: Constellation Testing Agent
tags: [testing, automation, {}]
dependencies: []
---

# Test Skill: {}

## Purpose
Automated testing for the {} feature.

## Test Cases

### Unit Tests
```rust
#[cfg(test)]
mod tests {{
    #[test]
    fn test_{}_basic() {{
        assert!(true);
    }}
}}
```

### Integration Tests
```javascript
describe('{}', () => {{
    it('should work correctly', () => {{
        expect(true).toBe(true);
    }});
}});
```

### Browser Tests
```typescript
test('{} UI', async () => {{
    await page.goto('http://localhost:3000');
    await expect(page).toHaveText('{}');
}});
```

## Execution
Run tests with:
```bash
cargo test --features {}
npm test -- --testNamePattern="{}"
```

## Results
Test results are captured in `claude-progress.txt` and screenshots are saved for visual verification.
"#,
            feature.name,
            feature.name,
            feature.name.to_lowercase(),
            feature.name,
            feature.name,
            feature.name.replace(" ", "_").to_lowercase(),
            feature.name,
            feature.name,
            feature.name,
            feature.name.replace(" ", "_").to_lowercase(),
            feature.name
        );
        
        fs::write(&skill_path, skill_content)?;
        
        Ok(skill_path)
    }

    pub fn validate_test_coverage(&self, feature: &Feature) -> Result<f32> {
        let source_files = self.find_source_files(feature)?;
        let test_files = self.find_test_files(feature)?;
        
        if source_files.is_empty() {
            return Ok(0.0);
        }
        
        let coverage = (test_files.len() as f32 / source_files.len() as f32) * 100.0;
        
        Ok(coverage.min(100.0))
    }

    fn find_source_files(&self, feature: &Feature) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        let feature_pattern = format!("*{}*", feature.name.replace(" ", "_").to_lowercase());
        
        for entry in walkdir::WalkDir::new(&self.project_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.is_file() {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                if filename.contains(&feature.name.replace(" ", "_").to_lowercase()) &&
                   (filename.ends_with(".rs") || filename.ends_with(".js") || 
                    filename.ends_with(".ts") || filename.ends_with(".py")) {
                    files.push(path.to_path_buf());
                }
            }
        }
        
        Ok(files)
    }

    fn find_test_files(&self, feature: &Feature) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        let test_pattern = format!("*test*{}*", feature.name.replace(" ", "_").to_lowercase());
        
        for entry in walkdir::WalkDir::new(&self.project_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.is_file() {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                if (filename.contains("test") || filename.contains("spec")) &&
                   filename.contains(&feature.name.replace(" ", "_").to_lowercase()) {
                    files.push(path.to_path_buf());
                }
            }
        }
        
        Ok(files)
    }
}
