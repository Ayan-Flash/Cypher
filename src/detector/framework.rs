#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

/// Detected frameworks in a codebase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Framework {
    // Web Frameworks
    React,
    Vue,
    Angular,
    Svelte,
    NextJs,
    Nuxt,
    Express,
    Django,
    Flask,
    FastAPI,
    Spring,
    Rails,
    Laravel,
    Phoenix,
    
    // Database Frameworks
    Sequelize,
    TypeORM,
    Prisma,
    Hibernate,
    SQLAlchemy,
    
    // Testing Frameworks
    Jest,
    Mocha,
    Pytest,
    RSpec,
    
    // Build Tools
    Webpack,
    Vite,
    Parcel,
    Gradle,
    Maven,
    
    // Other
    Docker,
    Kubernetes,
    Terraform,
}

impl Framework {
    /// Get all frameworks
    pub fn all() -> Vec<Self> {
        vec![
            Framework::React,
            Framework::Vue,
            Framework::Angular,
            Framework::Svelte,
            Framework::NextJs,
            Framework::Nuxt,
            Framework::Express,
            Framework::Django,
            Framework::Flask,
            Framework::FastAPI,
            Framework::Spring,
            Framework::Rails,
            Framework::Laravel,
            Framework::Phoenix,
            Framework::Sequelize,
            Framework::TypeORM,
            Framework::Prisma,
            Framework::Hibernate,
            Framework::SQLAlchemy,
            Framework::Jest,
            Framework::Mocha,
            Framework::Pytest,
            Framework::RSpec,
            Framework::Webpack,
            Framework::Vite,
            Framework::Parcel,
            Framework::Gradle,
            Framework::Maven,
            Framework::Docker,
            Framework::Kubernetes,
            Framework::Terraform,
        ]
    }

    /// Get detection patterns for this framework
    pub fn detection_patterns(&self) -> Vec<&'static str> {
        match self {
            Framework::React => vec![
                "package.json",
                "react",
                "jsx",
                "tsx",
                "ReactDOM",
            ],
            Framework::Vue => vec![
                "package.json",
                "vue",
                ".vue",
                "VueRouter",
            ],
            Framework::Angular => vec![
                "angular.json",
                "@angular",
                ".component.ts",
            ],
            Framework::Svelte => vec![
                "package.json",
                "svelte",
                ".svelte",
            ],
            Framework::NextJs => vec![
                "next.config.js",
                "next.config.ts",
                "pages/",
                "app/",
            ],
            Framework::Nuxt => vec![
                "nuxt.config.js",
                "nuxt.config.ts",
                "pages/",
            ],
            Framework::Express => vec![
                "package.json",
                "express",
                "app.js",
                "server.js",
            ],
            Framework::Django => vec![
                "manage.py",
                "settings.py",
                "wsgi.py",
                "urls.py",
            ],
            Framework::Flask => vec![
                "app.py",
                "wsgi.py",
                "requirements.txt",
                "Flask",
            ],
            Framework::FastAPI => vec![
                "main.py",
                "requirements.txt",
                "fastapi",
            ],
            Framework::Spring => vec![
                "pom.xml",
                "build.gradle",
                "Application.java",
                "@SpringBootApplication",
            ],
            Framework::Rails => vec![
                "Gemfile",
                "config/routes.rb",
                "app/",
                "Rakefile",
            ],
            Framework::Laravel => vec![
                "composer.json",
                "artisan",
                "app/",
                "routes/",
            ],
            Framework::Phoenix => vec![
                "mix.exs",
                "config/",
                "lib/",
            ],
            Framework::Sequelize => vec![
                "package.json",
                "sequelize",
                ".sequelize",
            ],
            Framework::TypeORM => vec![
                "package.json",
                "typeorm",
                "@typeorm",
            ],
            Framework::Prisma => vec![
                "prisma/",
                "schema.prisma",
            ],
            Framework::Hibernate => vec![
                "pom.xml",
                "hibernate",
                "hibernate.cfg.xml",
            ],
            Framework::SQLAlchemy => vec![
                "requirements.txt",
                "sqlalchemy",
                "SQLAlchemy",
            ],
            Framework::Jest => vec![
                "package.json",
                "jest.config.js",
                "*.test.js",
                "*.spec.js",
            ],
            Framework::Mocha => vec![
                "package.json",
                "mocha",
                "*.test.js",
            ],
            Framework::Pytest => vec![
                "pytest.ini",
                "conftest.py",
                "test_*.py",
            ],
            Framework::RSpec => vec![
                "Gemfile",
                "rspec",
                "_spec.rb",
            ],
            Framework::Webpack => vec![
                "webpack.config.js",
                "webpack.config.ts",
            ],
            Framework::Vite => vec![
                "vite.config.js",
                "vite.config.ts",
            ],
            Framework::Parcel => vec![
                ".parcelrc",
                "parcel",
            ],
            Framework::Gradle => vec![
                "build.gradle",
                "build.gradle.kts",
                "gradlew",
            ],
            Framework::Maven => vec![
                "pom.xml",
            ],
            Framework::Docker => vec![
                "Dockerfile",
                "docker-compose.yml",
            ],
            Framework::Kubernetes => vec![
                "k8s/",
                "kubernetes/",
                "*.yaml",
                "*.yml",
            ],
            Framework::Terraform => vec![
                "*.tf",
                "main.tf",
                "variables.tf",
            ],
        }
    }

    /// Get framework category
    pub fn category(&self) -> &'static str {
        match self {
            Framework::React | Framework::Vue | Framework::Angular | Framework::Svelte | Framework::NextJs | Framework::Nuxt => "web-framework",
            Framework::Express | Framework::Django | Framework::Flask | Framework::FastAPI | Framework::Spring | Framework::Rails | Framework::Laravel | Framework::Phoenix => "backend-framework",
            Framework::Sequelize | Framework::TypeORM | Framework::Prisma | Framework::Hibernate | Framework::SQLAlchemy => "orm",
            Framework::Jest | Framework::Mocha | Framework::Pytest | Framework::RSpec => "testing",
            Framework::Webpack | Framework::Vite | Framework::Parcel | Framework::Gradle | Framework::Maven => "build-tool",
            Framework::Docker | Framework::Kubernetes | Framework::Terraform => "infrastructure",
        }
    }
}

impl fmt::Display for Framework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Framework::React => write!(f, "React"),
            Framework::Vue => write!(f, "Vue"),
            Framework::Angular => write!(f, "Angular"),
            Framework::Svelte => write!(f, "Svelte"),
            Framework::NextJs => write!(f, "Next.js"),
            Framework::Nuxt => write!(f, "Nuxt"),
            Framework::Express => write!(f, "Express"),
            Framework::Django => write!(f, "Django"),
            Framework::Flask => write!(f, "Flask"),
            Framework::FastAPI => write!(f, "FastAPI"),
            Framework::Spring => write!(f, "Spring"),
            Framework::Rails => write!(f, "Rails"),
            Framework::Laravel => write!(f, "Laravel"),
            Framework::Phoenix => write!(f, "Phoenix"),
            Framework::Sequelize => write!(f, "Sequelize"),
            Framework::TypeORM => write!(f, "TypeORM"),
            Framework::Prisma => write!(f, "Prisma"),
            Framework::Hibernate => write!(f, "Hibernate"),
            Framework::SQLAlchemy => write!(f, "SQLAlchemy"),
            Framework::Jest => write!(f, "Jest"),
            Framework::Mocha => write!(f, "Mocha"),
            Framework::Pytest => write!(f, "Pytest"),
            Framework::RSpec => write!(f, "RSpec"),
            Framework::Webpack => write!(f, "Webpack"),
            Framework::Vite => write!(f, "Vite"),
            Framework::Parcel => write!(f, "Parcel"),
            Framework::Gradle => write!(f, "Gradle"),
            Framework::Maven => write!(f, "Maven"),
            Framework::Docker => write!(f, "Docker"),
            Framework::Kubernetes => write!(f, "Kubernetes"),
            Framework::Terraform => write!(f, "Terraform"),
        }
    }
}
