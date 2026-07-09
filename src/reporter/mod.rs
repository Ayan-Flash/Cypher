#![allow(dead_code, unused_imports)]

mod json;
mod sarif;
mod text;
mod html;

pub use json::JsonReporter;
pub use sarif::SarifReporter;
pub use text::TextReporter;
pub use html::HtmlReporter;

use crate::error::{Result, CypherError};
use crate::rules::RuleResult;

/// Reporter trait for generating security reports
pub trait Reporter {
    /// Generate a report from rule results
    fn generate(&self, results: &[RuleResult]) -> Result<String>;
    
    /// Get the reporter format name
    fn format_name(&self) -> &str;
}

/// Create a reporter by format name
pub fn create_reporter(format: &str) -> Result<Box<dyn Reporter>> {
    match format.to_lowercase().as_str() {
        "json" => Ok(Box::new(JsonReporter::new())),
        "sarif" => Ok(Box::new(SarifReporter::new())),
        "text" => Ok(Box::new(TextReporter::new())),
        "html" => Ok(Box::new(HtmlReporter::new())),
        _ => Err(CypherError::InvalidOutputFormat(format.to_string())),
    }
}
