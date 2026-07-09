use crate::error::Result;
use crate::rules::RuleResult;
use crate::reporter::Reporter;
use serde_json::json;

/// SARIF (Static Analysis Results Interchange Format) reporter
pub struct SarifReporter;

impl SarifReporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SarifReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for SarifReporter {
    fn generate(&self, results: &[RuleResult]) -> Result<String> {
        let mut rules = Vec::new();
        let mut results_vec = Vec::new();

        for result in results {
            // Add rule definition if not already present
            if !rules.iter().any(|r: &serde_json::Value| r["id"] == result.rule.id) {
                rules.push(json!({
                    "id": result.rule.id,
                    "name": result.rule.name,
                    "shortDescription": {
                        "text": result.rule.description
                    },
                    "fullDescription": {
                        "text": result.rule.description
                    },
                    "help": {
                        "text": format!("{}: {}", result.rule.category, result.rule.description)
                    },
                    "properties": {
                        "category": result.rule.category.to_string(),
                        "severity": result.rule.severity.to_string()
                    }
                }));

                if let Some(cwe) = &result.rule.cwe {
                    rules.last_mut().unwrap()["properties"]["cwe"] = json!(cwe);
                }
                if let Some(owasp) = &result.rule.owasp {
                    rules.last_mut().unwrap()["properties"]["owasp"] = json!(owasp);
                }
            }

            // Add results for each match
            for match_ in &result.matches {
                results_vec.push(json!({
                    "ruleId": result.rule.id,
                    "level": sarif_level(&result.rule.severity),
                    "message": {
                        "text": format!("{}: {}", result.rule.name, result.rule.description)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": match_.file.to_string_lossy()
                            },
                            "region": {
                                "startLine": match_.line as i64,
                                "startColumn": match_.column as i64,
                                "snippet": {
                                    "text": match_.snippet
                                }
                            }
                        }
                    }]
                }));
            }
        }

        let sarif = json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "Cypher CLI",
                        "version": "0.1.0",
                        "informationUri": "https://github.com/Ayan-Flash/Cypher",
                        "rules": rules
                    }
                },
                "results": results_vec
            }]
        });

        Ok(serde_json::to_string_pretty(&sarif)?)
    }

    fn format_name(&self) -> &str {
        "sarif"
    }
}

fn sarif_level(severity: &crate::rules::Severity) -> &str {
    match severity.to_string().as_str() {
        "critical" => "error",
        "high" => "error",
        "medium" => "warning",
        "low" => "note",
        _ => "note",
    }
}
