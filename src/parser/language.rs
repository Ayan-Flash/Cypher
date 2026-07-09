use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Supported programming languages for AST parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Java,
    C,
    Cpp,
    Csharp,
    Php,
    Ruby,
    Swift,
    Kotlin,
    Scala,
}

impl Language {
    /// Get all supported languages
    pub fn all() -> Vec<Self> {
        vec![
            Language::Rust,
            Language::JavaScript,
            Language::TypeScript,
            Language::Python,
            Language::Go,
            Language::Java,
            Language::C,
            Language::Cpp,
            Language::Csharp,
            Language::Php,
            Language::Ruby,
            Language::Swift,
            Language::Kotlin,
            Language::Scala,
        ]
    }

    /// Get file extensions for this language
    pub fn extensions(&self) -> Vec<&'static str> {
        match self {
            Language::Rust => vec!["rs"],
            Language::JavaScript => vec!["js", "jsx", "mjs"],
            Language::TypeScript => vec!["ts", "tsx"],
            Language::Python => vec!["py"],
            Language::Go => vec!["go"],
            Language::Java => vec!["java"],
            Language::C => vec!["c", "h"],
            Language::Cpp => vec!["cpp", "hpp", "cc", "cxx", "hxx"],
            Language::Csharp => vec!["cs"],
            Language::Php => vec!["php"],
            Language::Ruby => vec!["rb"],
            Language::Swift => vec!["swift"],
            Language::Kotlin => vec!["kt", "kts"],
            Language::Scala => vec!["scala"],
        }
    }

    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext_lower = ext.to_lowercase();
        for lang in Self::all() {
            if lang.extensions().contains(&ext_lower.as_str()) {
                return Some(lang);
            }
        }
        None
    }

    /// Get tree-sitter language name
    pub fn tree_sitter_name(&self) -> &str {
        match self {
            Language::Rust => "rust",
            Language::JavaScript => "javascript",
            Language::TypeScript => "typescript",
            Language::Python => "python",
            Language::Go => "go",
            Language::Java => "java",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Csharp => "c_sharp",
            Language::Php => "php",
            Language::Ruby => "ruby",
            Language::Swift => "swift",
            Language::Kotlin => "kotlin",
            Language::Scala => "scala",
        }
    }
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" => Ok(Language::Rust),
            "javascript" | "js" => Ok(Language::JavaScript),
            "typescript" | "ts" => Ok(Language::TypeScript),
            "python" | "py" => Ok(Language::Python),
            "go" => Ok(Language::Go),
            "java" => Ok(Language::Java),
            "c" => Ok(Language::C),
            "cpp" | "c++" => Ok(Language::Cpp),
            "csharp" | "c#" => Ok(Language::Csharp),
            "php" => Ok(Language::Php),
            "ruby" | "rb" => Ok(Language::Ruby),
            "swift" => Ok(Language::Swift),
            "kotlin" | "kt" => Ok(Language::Kotlin),
            "scala" => Ok(Language::Scala),
            _ => Err(format!("Unsupported language: {}", s)),
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Rust => write!(f, "rust"),
            Language::JavaScript => write!(f, "javascript"),
            Language::TypeScript => write!(f, "typescript"),
            Language::Python => write!(f, "python"),
            Language::Go => write!(f, "go"),
            Language::Java => write!(f, "java"),
            Language::C => write!(f, "c"),
            Language::Cpp => write!(f, "cpp"),
            Language::Csharp => write!(f, "csharp"),
            Language::Php => write!(f, "php"),
            Language::Ruby => write!(f, "ruby"),
            Language::Swift => write!(f, "swift"),
            Language::Kotlin => write!(f, "kotlin"),
            Language::Scala => write!(f, "scala"),
        }
    }
}
