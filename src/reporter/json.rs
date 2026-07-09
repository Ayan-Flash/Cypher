#![allow(dead_code)]

use crate::error::Result;
use crate::rules::RuleResult;
use crate::reporter::Reporter;
use serde_json::json;

/// JSON reporter for machine-readable output
pub struct JsonReporter {
    pretty: bool,
}

impl JsonReporter {
    pub fn new() -> Self {
        Self { pretty: true }
    }

    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }
}

impl Default for JsonReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for JsonReporter {
    fn generate(&self, results: &[RuleResult]) -> Result<String> {
        let mut findings = Vec::new();
        let mut summary = std::collections::HashMap::new();
        
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;

        for result in results {
            let severity = result.rule.severity.to_string();
            match severity.as_str() {
                "critical" => critical_count += result.finding_count(),
                "high" => high_count += result.finding_count(),
                "medium" => medium_count += result.finding_count(),
                "low" => low_count += result.finding_count(),
                _ => {}
            }

            for match_ in &result.matches {
                findings.push(json!({
                    "rule_id": result.rule.id,
                    "rule_name": result.rule.name,
                    "severity": severity,
                    "category": result.rule.category.to_string(),
                    "description": result.rule.description,
                    "cwe": result.rule.cwe,
                    "owasp": result.rule.owasp,
                    "file": match_.file,
                    "line": match_.line,
                    "column": match_.column,
                    "snippet": match_.snippet,
                    "context": match_.context,
                }));
            }
        }

        summary.insert("critical", critical_count);
        summary.insert("high", high_count);
        summary.insert("medium", medium_count);
        summary.insert("low", low_count);
        summary.insert("total", critical_count + high_count + medium_count + low_count);

        let report = json!({
            "version": "0.1.0",
            "tool": "cypher-cli",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "summary": summary,
            "findings": findings,
        });

        if self.pretty {
            Ok(serde_json::to_string_pretty(&report)?)
        } else {
            Ok(serde_json::to_string(&report)?)
        }
    }

    fn format_name(&self) -> &str {
        "json"
    }
}
