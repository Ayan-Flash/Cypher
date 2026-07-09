use crate::error::Result;
use crate::rules::RuleResult;
use crate::reporter::Reporter;
use colored::Colorize;

/// Text reporter for console output
pub struct TextReporter {
    include_snippets: bool,
}

impl TextReporter {
    pub fn new() -> Self {
        Self {
            include_snippets: true,
        }
    }

    #[allow(dead_code)]
    pub fn with_snippets(mut self, include: bool) -> Self {
        self.include_snippets = include;
        self
    }
}

impl Default for TextReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for TextReporter {
    fn generate(&self, results: &[RuleResult]) -> Result<String> {
        let mut output = String::new();
        
        // Header
        output.push_str(&format!(
            "{}\n{}\n",
            "Cypher Security Report".bold().white(),
            "=".repeat(80).dimmed()
        ));
        output.push_str(&format!("Total findings: {}\n\n", total_findings(results)));

        // Group by severity
        let critical = results_by_severity(results, "critical");
        let high = results_by_severity(results, "high");
        let medium = results_by_severity(results, "medium");
        let low = results_by_severity(results, "low");

        // Print findings by severity
        if !critical.is_empty() {
            output.push_str(&format!("\n{}\n", "CRITICAL".red().bold()));
            output.push_str(&format!("{}\n\n", "=".repeat(80).red()));
            for result in &critical {
                output.push_str(&self.format_result(result));
            }
        }

        if !high.is_empty() {
            output.push_str(&format!("\n{}\n", "HIGH".yellow().bold()));
            output.push_str(&format!("{}\n\n", "=".repeat(80).yellow()));
            for result in &high {
                output.push_str(&self.format_result(result));
            }
        }

        if !medium.is_empty() {
            output.push_str(&format!("\n{}\n", "MEDIUM".blue().bold()));
            output.push_str(&format!("{}\n\n", "=".repeat(80).blue()));
            for result in &medium {
                output.push_str(&self.format_result(result));
            }
        }

        if !low.is_empty() {
            output.push_str(&format!("\n{}\n", "LOW".green().bold()));
            output.push_str(&format!("{}\n\n", "=".repeat(80).green()));
            for result in &low {
                output.push_str(&self.format_result(result));
            }
        }

        // Summary
        output.push_str(&format!(
            "\n{}\n",
            "=".repeat(80).dimmed()
        ));
        output.push_str("Summary:\n");
        output.push_str(&format!("  Critical: {}\n", critical.len()));
        output.push_str(&format!("  High: {}\n", high.len()));
        output.push_str(&format!("  Medium: {}\n", medium.len()));
        output.push_str(&format!("  Low: {}\n", low.len()));
        output.push_str(&format!("  Total: {}\n", total_findings(results)));

        Ok(output)
    }

    fn format_name(&self) -> &str {
        "text"
    }
}

impl TextReporter {
    fn format_result(&self, result: &RuleResult) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("{}: {}\n", 
            result.rule.id.bold(),
            result.rule.name
        ));
        output.push_str(&format!("  Severity: {}\n", 
            format_severity(&result.rule.severity)
        ));
        output.push_str(&format!("  Category: {}\n", 
            result.rule.category
        ));
        output.push_str(&format!("  Description: {}\n", 
            result.rule.description
        ));

        if let Some(cwe) = &result.rule.cwe {
            output.push_str(&format!("  CWE: {}\n", cwe));
        }

        if let Some(owasp) = &result.rule.owasp {
            output.push_str(&format!("  OWASP: {}\n", owasp));
        }

        output.push_str(&format!("  Findings: {}\n", result.finding_count()));

        if self.include_snippets && !result.matches.is_empty() {
            output.push_str("\n  Matches:\n");
            for (i, match_) in result.matches.iter().enumerate().take(5) {
                output.push_str(&format!("    {}. {}:{}:{}\n", 
                    i + 1,
                    match_.file.display(),
                    match_.line,
                    match_.column
                ));
                output.push_str(&format!("       {}\n", match_.snippet.dimmed()));
                if !match_.context.is_empty() {
                    output.push_str(&format!("       {}\n", match_.context.dimmed()));
                }
            }
            if result.matches.len() > 5 {
                output.push_str(&format!("    ... and {} more\n", result.matches.len() - 5));
            }
        }

        output.push('\n');
        output
    }
}

fn total_findings(results: &[RuleResult]) -> usize {
    results.iter().map(|r| r.finding_count()).sum()
}

fn results_by_severity<'a>(results: &'a [RuleResult], severity: &str) -> Vec<&'a RuleResult> {
    results
        .iter()
        .filter(|r| r.rule.severity.to_string() == severity)
        .collect()
}

fn format_severity(severity: &crate::rules::Severity) -> colored::ColoredString {
    match severity.to_string().as_str() {
        "critical" => "CRITICAL".red().bold(),
        "high" => "HIGH".yellow().bold(),
        "medium" => "MEDIUM".blue().bold(),
        "low" => "LOW".green().bold(),
        _ => severity.to_string().normal(),
    }
}
