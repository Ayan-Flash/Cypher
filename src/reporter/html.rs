use crate::error::Result;
use crate::rules::RuleResult;
use crate::reporter::Reporter;

/// HTML reporter for human-readable web reports
pub struct HtmlReporter;

impl HtmlReporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for HtmlReporter {
    fn generate(&self, results: &[RuleResult]) -> Result<String> {
        let mut html = String::new();
        
        // HTML header
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str("    <title>Cypher Security Report</title>\n");
        html.push_str("    <style>\n");
        html.push_str("        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }\n");
        html.push_str("        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }\n");
        html.push_str("        h1 { color: #333; border-bottom: 2px solid #007bff; padding-bottom: 10px; }\n");
        html.push_str("        .summary { display: flex; gap: 20px; margin: 20px 0; }\n");
        html.push_str("        .summary-card { flex: 1; padding: 20px; border-radius: 6px; color: white; }\n");
        html.push_str("        .critical { background: #dc3545; }\n");
        html.push_str("        .high { background: #ffc107; color: #333; }\n");
        html.push_str("        .medium { background: #17a2b8; }\n");
        html.push_str("        .low { background: #28a745; }\n");
        html.push_str("        .finding { margin: 20px 0; padding: 20px; border-left: 4px solid #007bff; background: #f8f9fa; }\n");
        html.push_str("        .finding.critical { border-left-color: #dc3545; }\n");
        html.push_str("        .finding.high { border-left-color: #ffc107; }\n");
        html.push_str("        .finding.medium { border-left-color: #17a2b8; }\n");
        html.push_str("        .finding.low { border-left-color: #28a745; }\n");
        html.push_str("        .finding h3 { margin-top: 0; color: #333; }\n");
        html.push_str("        .severity { display: inline-block; padding: 4px 8px; border-radius: 4px; font-size: 12px; font-weight: bold; }\n");
        html.push_str("        .severity.critical { background: #dc3545; color: white; }\n");
        html.push_str("        .severity.high { background: #ffc107; color: #333; }\n");
        html.push_str("        .severity.medium { background: #17a2b8; color: white; }\n");
        html.push_str("        .severity.low { background: #28a745; color: white; }\n");
        html.push_str("        .code-block { background: #2d2d2d; color: #f8f8f2; padding: 15px; border-radius: 4px; overflow-x: auto; margin: 10px 0; }\n");
        html.push_str("        .match { margin: 10px 0; padding: 10px; background: white; border-radius: 4px; }\n");
        html.push_str("        .match-file { font-weight: bold; color: #007bff; }\n");
        html.push_str("        .match-location { color: #666; font-size: 14px; }\n");
        html.push_str("        .footer { margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; color: #666; text-align: center; }\n");
        html.push_str("    </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        html.push_str("    <div class=\"container\">\n");
        
        // Header
        html.push_str("        <h1>🛡️ Cypher Security Report</h1>\n");
        html.push_str(&format!("        <p>Generated: {}</p>\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        // Summary
        let critical_count = count_by_severity(results, "critical");
        let high_count = count_by_severity(results, "high");
        let medium_count = count_by_severity(results, "medium");
        let low_count = count_by_severity(results, "low");
        let total = critical_count + high_count + medium_count + low_count;
        
        html.push_str("        <div class=\"summary\">\n");
        html.push_str(&format!("            <div class=\"summary-card critical\"><h3>{}</h3><p>Critical</p></div>\n", critical_count));
        html.push_str(&format!("            <div class=\"summary-card high\"><h3>{}</h3><p>High</p></div>\n", high_count));
        html.push_str(&format!("            <div class=\"summary-card medium\"><h3>{}</h3><p>Medium</p></div>\n", medium_count));
        html.push_str(&format!("            <div class=\"summary-card low\"><h3>{}</h3><p>Low</p></div>\n", low_count));
        html.push_str("        </div>\n");
        html.push_str(&format!("        <p><strong>Total Findings:</strong> {}</p>\n", total));
        
        // Findings
        html.push_str("        <h2>Findings</h2>\n");
        
        for result in results {
            if result.matches.is_empty() {
                continue;
            }
            
            let severity_class = result.rule.severity.to_string();
            html.push_str(&format!("        <div class=\"finding {}\">\n", severity_class));
            html.push_str(&format!("            <h3>{} - {}</h3>\n", result.rule.id, result.rule.name));
            html.push_str(&format!("            <span class=\"severity {}\">{}</span>\n", severity_class, result.rule.severity.to_string().to_uppercase()));
            html.push_str(&format!("            <p><strong>Category:</strong> {}</p>\n", result.rule.category));
            html.push_str(&format!("            <p><strong>Description:</strong> {}</p>\n", result.rule.description));
            
            if let Some(cwe) = &result.rule.cwe {
                html.push_str(&format!("            <p><strong>CWE:</strong> {}</p>\n", cwe));
            }
            
            if let Some(owasp) = &result.rule.owasp {
                html.push_str(&format!("            <p><strong>OWASP:</strong> {}</p>\n", owasp));
            }
            
            html.push_str(&format!("            <p><strong>Matches:</strong> {}</p>\n", result.matches.len()));
            
            // Show first 5 matches
            for (_i, match_) in result.matches.iter().enumerate().take(5) {
                html.push_str("            <div class=\"match\">\n");
                html.push_str(&format!("                <div class=\"match-file\">{}</div>\n", match_.file.display()));
                html.push_str(&format!("                <div class=\"match-location\">Line {}, Column {}</div>\n", match_.line, match_.column));
                html.push_str("                <div class=\"code-block\">\n");
                html.push_str(&format!("                    <code>{}</code>\n", html_escape(&match_.snippet)));
                html.push_str("                </div>\n");
                html.push_str("            </div>\n");
            }
            
            if result.matches.len() > 5 {
                html.push_str(&format!("            <p>... and {} more matches</p>\n", result.matches.len() - 5));
            }
            
            html.push_str("        </div>\n");
        }
        
        // Footer
        html.push_str("        <div class=\"footer\">\n");
        html.push_str("            <p>Generated by Cypher CLI v0.1.0</p>\n");
        html.push_str("            <p><a href=\"https://github.com/sentinel-security/cypher-cli\">https://github.com/sentinel-security/cypher-cli</a></p>\n");
        html.push_str("        </div>\n");
        html.push_str("    </div>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");
        
        Ok(html)
    }

    fn format_name(&self) -> &str {
        "html"
    }
}

fn count_by_severity(results: &[RuleResult], severity: &str) -> usize {
    results
        .iter()
        .filter(|r| r.rule.severity.to_string() == severity)
        .map(|r| r.finding_count())
        .sum()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
