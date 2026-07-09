#![allow(dead_code)]

use crate::detector::Framework;
use crate::error::{Result, CypherError};
use std::path::Path;
use walkdir::WalkDir;

/// Framework detector for identifying frameworks in a codebase
pub struct Detector {
    detected_frameworks: Vec<Framework>,
}

impl Detector {
    /// Create a new detector
    pub fn new() -> Self {
        Self {
            detected_frameworks: Vec::new(),
        }
    }

    /// Detect frameworks in a directory
    pub fn detect(&mut self, path: &Path) -> Result<Vec<Framework>> {
        self.detected_frameworks.clear();

        if !path.exists() {
            return Err(CypherError::PathNotFound(path.display().to_string()));
        }

        // Check for framework-specific files
        self.check_files(path)?;

        // Check package.json for JavaScript frameworks
        if let Some(package_json) = path.join("package.json").canonicalize().ok() {
            if package_json.exists() {
                self.check_package_json(&package_json)?;
            }
        }

        // Check requirements.txt for Python frameworks
        if let Some(requirements) = path.join("requirements.txt").canonicalize().ok() {
            if requirements.exists() {
                self.check_requirements_txt(&requirements)?;
            }
        }

        // Check Gemfile for Ruby frameworks
        if let Some(gemfile) = path.join("Gemfile").canonicalize().ok() {
            if gemfile.exists() {
                self.check_gemfile(&gemfile)?;
            }
        }

        // Check pom.xml for Java frameworks
        if let Some(pom) = path.join("pom.xml").canonicalize().ok() {
            if pom.exists() {
                self.check_pom_xml(&pom)?;
            }
        }

        // Check build.gradle for Java/Kotlin frameworks
        if let Some(gradle) = path.join("build.gradle").canonicalize().ok() {
            if gradle.exists() {
                self.check_gradle(&gradle)?;
            }
        }

        // Check composer.json for PHP frameworks
        if let Some(composer) = path.join("composer.json").canonicalize().ok() {
            if composer.exists() {
                self.check_composer_json(&composer)?;
            }
        }

        // Check mix.exs for Elixir frameworks
        if let Some(mix) = path.join("mix.exs").canonicalize().ok() {
            if mix.exists() {
                self.check_mix_exs(&mix)?;
            }
        }

        // Check for Docker files
        self.check_docker(path)?;

        // Check for Terraform files
        self.check_terraform(path)?;

        // Check for Kubernetes files
        self.check_kubernetes(path)?;

        Ok(self.detected_frameworks.clone())
    }

    /// Check for framework-specific files
    fn check_files(&mut self, path: &Path) -> Result<()> {
        for entry in WalkDir::new(path).max_depth(2).into_iter().filter_map(|e| e.ok()) {
            let file_name = entry.file_name().to_string_lossy().to_lowercase();
            
            for framework in Framework::all() {
                for pattern in framework.detection_patterns() {
                    if file_name.contains(&pattern.to_lowercase()) {
                        if !self.detected_frameworks.contains(&framework) {
                            self.detected_frameworks.push(framework);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Check package.json for JavaScript frameworks
    fn check_package_json(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        
        let frameworks_to_check = vec![
            (Framework::React, "react"),
            (Framework::Vue, "vue"),
            (Framework::Angular, "@angular"),
            (Framework::Svelte, "svelte"),
            (Framework::Express, "express"),
            (Framework::Jest, "jest"),
            (Framework::Mocha, "mocha"),
            (Framework::Webpack, "webpack"),
            (Framework::Vite, "vite"),
            (Framework::Parcel, "parcel"),
            (Framework::Sequelize, "sequelize"),
            (Framework::TypeORM, "typeorm"),
            (Framework::Prisma, "prisma"),
        ];

        for (framework, keyword) in frameworks_to_check {
            if content.contains(keyword) {
                if !self.detected_frameworks.contains(&framework) {
                    self.detected_frameworks.push(framework);
                }
            }
        }

        Ok(())
    }

    /// Check requirements.txt for Python frameworks
    fn check_requirements_txt(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        
        let frameworks_to_check = vec![
            (Framework::Django, "django"),
            (Framework::Flask, "flask"),
            (Framework::FastAPI, "fastapi"),
            (Framework::Pytest, "pytest"),
            (Framework::SQLAlchemy, "sqlalchemy"),
        ];

        for (framework, keyword) in frameworks_to_check {
            if content.to_lowercase().contains(keyword) {
                if !self.detected_frameworks.contains(&framework) {
                    self.detected_frameworks.push(framework);
                }
            }
        }

        Ok(())
    }

    /// Check Gemfile for Ruby frameworks
    fn check_gemfile(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        
        let frameworks_to_check = vec![
            (Framework::Rails, "rails"),
            (Framework::RSpec, "rspec"),
        ];

        for (framework, keyword) in frameworks_to_check {
            if content.to_lowercase().contains(keyword) {
                if !self.detected_frameworks.contains(&framework) {
                    self.detected_frameworks.push(framework);
                }
            }
        }

        Ok(())
    }

    /// Check pom.xml for Java frameworks
    fn check_pom_xml(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        
        let frameworks_to_check = vec![
            (Framework::Spring, "spring"),
            (Framework::Hibernate, "hibernate"),
            (Framework::Maven, "maven"),
        ];

        for (framework, keyword) in frameworks_to_check {
            if content.to_lowercase().contains(keyword) {
                if !self.detected_frameworks.contains(&framework) {
                    self.detected_frameworks.push(framework);
                }
            }
        }

        Ok(())
    }

    /// Check build.gradle for Java/Kotlin frameworks
    fn check_gradle(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        
        let frameworks_to_check = vec![
            (Framework::Spring, "spring"),
            (Framework::Gradle, "gradle"),
        ];

        for (framework, keyword) in frameworks_to_check {
            if content.to_lowercase().contains(keyword) {
                if !self.detected_frameworks.contains(&framework) {
                    self.detected_frameworks.push(framework);
                }
            }
        }

        Ok(())
    }

    /// Check composer.json for PHP frameworks
    fn check_composer_json(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        
        if content.to_lowercase().contains("laravel") {
            if !self.detected_frameworks.contains(&Framework::Laravel) {
                self.detected_frameworks.push(Framework::Laravel);
            }
        }

        Ok(())
    }

    /// Check mix.exs for Elixir frameworks
    fn check_mix_exs(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        
        if content.to_lowercase().contains("phoenix") {
            if !self.detected_frameworks.contains(&Framework::Phoenix) {
                self.detected_frameworks.push(Framework::Phoenix);
            }
        }

        Ok(())
    }

    /// Check for Docker files
    fn check_docker(&mut self, path: &Path) -> Result<()> {
        let dockerfile = path.join("Dockerfile");
        let docker_compose = path.join("docker-compose.yml");

        if dockerfile.exists() || docker_compose.exists() {
            if !self.detected_frameworks.contains(&Framework::Docker) {
                self.detected_frameworks.push(Framework::Docker);
            }
        }

        Ok(())
    }

    /// Check for Terraform files
    fn check_terraform(&mut self, path: &Path) -> Result<()> {
        for entry in WalkDir::new(path).max_depth(2).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("tf") {
                if !self.detected_frameworks.contains(&Framework::Terraform) {
                    self.detected_frameworks.push(Framework::Terraform);
                    break;
                }
            }
        }
        Ok(())
    }

    /// Check for Kubernetes files
    fn check_kubernetes(&mut self, path: &Path) -> Result<()> {
        let k8s_dirs = vec!["k8s", "kubernetes", ".k8s"];
        
        for dir_name in k8s_dirs {
            let dir = path.join(dir_name);
            if dir.exists() && dir.is_dir() {
                if !self.detected_frameworks.contains(&Framework::Kubernetes) {
                    self.detected_frameworks.push(Framework::Kubernetes);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Get detected frameworks
    pub fn frameworks(&self) -> &[Framework] {
        &self.detected_frameworks
    }

    /// Check if a specific framework is detected
    pub fn has_framework(&self, framework: Framework) -> bool {
        self.detected_frameworks.contains(&framework)
    }
}

impl Default for Detector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detector_creation() {
        let detector = Detector::new();
        assert!(detector.frameworks().is_empty());
    }

    #[test]
    fn test_detect_docker() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Dockerfile"), "FROM node:18").unwrap();

        let mut detector = Detector::new();
        let frameworks = detector.detect(temp_dir.path()).unwrap();
        
        assert!(frameworks.contains(&Framework::Docker));
    }

    #[test]
    fn test_detect_terraform() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("main.tf"), "resource \"aws_instance\" \"example\" {}").unwrap();

        let mut detector = Detector::new();
        let frameworks = detector.detect(temp_dir.path()).unwrap();
        
        assert!(frameworks.contains(&Framework::Terraform));
    }
}
