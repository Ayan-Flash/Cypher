use crate::rules::{Rule, RuleCategory, Severity};

/// Built-in security rules library
pub struct RuleLibrary;

impl RuleLibrary {
    /// Get all built-in security rules
    pub fn get_all_rules() -> Vec<Rule> {
        vec![
            // Hardcoded Secrets
            Self::hardcoded_api_key(),
            Self::hardcoded_password(),
            Self::hardcoded_token(),
            Self::hardcoded_private_key(),
            
            // SQL Injection
            Self::sql_injection_concat(),
            Self::sql_injection_format(),
            
            // Command Injection
            Self::command_execution(),
            Self::shell_command(),
            
            // Path Traversal
            Self::path_traversal(),
            
            // XSS
            Self::xss_inner_html(),
            Self::xss_document_write(),
            
            // Insecure Cryptography
            Self::weak_hash_md5(),
            Self::weak_hash_sha1(),
            Self::insecure_random(),
            
            // Hardcoded Credentials
            Self::hardcoded_aws_key(),
            Self::hardcoded_database_url(),
            
            // Insecure Deserialization
            Self::unsafe_deserialization(),
            
            // Sensitive Data Exposure
            Self::sensitive_data_logging(),
            
            // Authentication Issues
            Self::hardcoded_auth_token(),
        ]
    }

    /// Hardcoded API Key
    fn hardcoded_api_key() -> Rule {
        Rule::new(
            "SEC-001".to_string(),
            "Hardcoded API Key".to_string(),
            "API keys should not be hardcoded in source code. Use environment variables or secret management.".to_string(),
            Severity::Critical,
            RuleCategory::HardcodedSecrets,
            vec!["rust".to_string(), "javascript".to_string(), "python".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-798".to_string())
        .with_pattern("(?i)(api[_-]?key|apikey)\\s*[:=]\\s*['\"][a-zA-Z0-9]{20,}['\"]".to_string())
    }

    /// Hardcoded Password
    fn hardcoded_password() -> Rule {
        Rule::new(
            "SEC-002".to_string(),
            "Hardcoded Password".to_string(),
            "Passwords should not be hardcoded in source code. Use secure credential storage.".to_string(),
            Severity::Critical,
            RuleCategory::HardcodedSecrets,
            vec!["rust".to_string(), "javascript".to_string(), "python".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-798".to_string())
        .with_pattern("(?i)(password|passwd|pwd)\\s*[:=]\\s*['\"][^'\"]{6,}['\"]".to_string())
    }

    /// Hardcoded Token
    fn hardcoded_token() -> Rule {
        Rule::new(
            "SEC-003".to_string(),
            "Hardcoded Authentication Token".to_string(),
            "Authentication tokens should not be hardcoded. Use secure token management.".to_string(),
            Severity::Critical,
            RuleCategory::HardcodedSecrets,
            vec!["rust".to_string(), "javascript".to_string(), "python".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-798".to_string())
        .with_pattern("(?i)(token|auth[_-]?token|bearer[_-]?token)\\s*[:=]\\s*['\"][a-zA-Z0-9]{20,}['\"]".to_string())
    }

    /// Hardcoded Private Key
    fn hardcoded_private_key() -> Rule {
        Rule::new(
            "SEC-004".to_string(),
            "Hardcoded Private Key".to_string(),
            "Private keys should never be hardcoded in source code.".to_string(),
            Severity::Critical,
            RuleCategory::HardcodedSecrets,
            vec!["rust".to_string(), "javascript".to_string(), "python".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-798".to_string())
        .with_pattern(r"-----BEGIN (RSA )?PRIVATE KEY-----".to_string())
    }

    /// SQL Injection via String Concatenation
    fn sql_injection_concat() -> Rule {
        Rule::new(
            "SEC-005".to_string(),
            "SQL Injection via String Concatenation".to_string(),
            "SQL queries constructed via string concatenation are vulnerable to injection. Use parameterized queries.".to_string(),
            Severity::High,
            RuleCategory::SqlInjection,
            vec!["javascript".to_string(), "typescript".to_string(), "python".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-89".to_string())
        .with_owasp("A03:2021 - Injection".to_string())
        .with_pattern("(SELECT|INSERT|UPDATE|DELETE).*\\+.*".to_string())
    }

    /// SQL Injection via String Format
    fn sql_injection_format() -> Rule {
        Rule::new(
            "SEC-006".to_string(),
            "SQL Injection via String Format".to_string(),
            "SQL queries constructed via string formatting are vulnerable to injection. Use parameterized queries.".to_string(),
            Severity::High,
            RuleCategory::SqlInjection,
            vec!["python".to_string(), "javascript".to_string(), "rust".to_string()],
        )
        .with_cwe("CWE-89".to_string())
        .with_owasp("A03:2021 - Injection".to_string())
        .with_pattern("(SELECT|INSERT|UPDATE|DELETE).*(format|f\"|format!|\\{\\})".to_string())
    }

    /// Command Execution
    fn command_execution() -> Rule {
        Rule::new(
            "SEC-007".to_string(),
            "Unsafe Command Execution".to_string(),
            "Executing commands with user input can lead to command injection. Validate and sanitize all inputs.".to_string(),
            Severity::Critical,
            RuleCategory::CommandInjection,
            vec!["python".to_string(), "javascript".to_string(), "rust".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-78".to_string())
        .with_owasp("A03:2021 - Injection".to_string())
        .with_pattern("(exec|system|spawn|popen|shell_exec)\\s*\\(".to_string())
    }

    /// Shell Command
    fn shell_command() -> Rule {
        Rule::new(
            "SEC-008".to_string(),
            "Shell Command with User Input".to_string(),
            "Shell commands with user input are vulnerable to injection attacks.".to_string(),
            Severity::Critical,
            RuleCategory::CommandInjection,
            vec!["rust".to_string(), "go".to_string(), "python".to_string()],
        )
        .with_cwe("CWE-78".to_string())
        .with_pattern("Command::new|exec\\.Command|subprocess\\.call".to_string())
    }

    /// Path Traversal
    fn path_traversal() -> Rule {
        Rule::new(
            "SEC-009".to_string(),
            "Path Traversal Vulnerability".to_string(),
            "File operations without path validation can lead to path traversal attacks. Validate and sanitize file paths.".to_string(),
            Severity::High,
            RuleCategory::PathTraversal,
            vec!["rust".to_string(), "javascript".to_string(), "python".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-22".to_string())
        .with_owasp("A01:2021 - Broken Access Control".to_string())
        .with_pattern("\\.\\./|\\.\\\\|%2e%2e".to_string())
    }

    /// XSS via innerHTML
    fn xss_inner_html() -> Rule {
        Rule::new(
            "SEC-010".to_string(),
            "XSS via innerHTML Assignment".to_string(),
            "Setting innerHTML with user input can lead to XSS. Use textContent or sanitize input.".to_string(),
            Severity::High,
            RuleCategory::Xss,
            vec!["javascript".to_string(), "typescript".to_string()],
        )
        .with_cwe("CWE-79".to_string())
        .with_owasp("A03:2021 - Injection".to_string())
        .with_pattern("\\.innerHTML\\s*=".to_string())
    }

    /// XSS via document.write
    fn xss_document_write() -> Rule {
        Rule::new(
            "SEC-011".to_string(),
            "XSS via document.write".to_string(),
            "document.write with user input can lead to XSS. Avoid using document.write with untrusted data.".to_string(),
            Severity::High,
            RuleCategory::Xss,
            vec!["javascript".to_string(), "typescript".to_string()],
        )
        .with_cwe("CWE-79".to_string())
        .with_owasp("A03:2021 - Injection".to_string())
        .with_pattern("document\\.write\\s*\\(".to_string())
    }

    /// Weak Hash (MD5)
    fn weak_hash_md5() -> Rule {
        Rule::new(
            "SEC-012".to_string(),
            "Weak Hash Algorithm (MD5)".to_string(),
            "MD5 is cryptographically broken and should not be used for security purposes. Use SHA-256 or better.".to_string(),
            Severity::Medium,
            RuleCategory::Cryptography,
            vec!["rust".to_string(), "javascript".to_string(), "python".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-327".to_string())
        .with_owasp("A02:2021 - Cryptographic Failures".to_string())
        .with_pattern("(?i)md5".to_string())
    }

    /// Weak Hash (SHA1)
    fn weak_hash_sha1() -> Rule {
        Rule::new(
            "SEC-013".to_string(),
            "Weak Hash Algorithm (SHA1)".to_string(),
            "SHA-1 is deprecated for security purposes. Use SHA-256 or better.".to_string(),
            Severity::Medium,
            RuleCategory::Cryptography,
            vec!["rust".to_string(), "javascript".to_string(), "python".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-327".to_string())
        .with_owasp("A02:2021 - Cryptographic Failures".to_string())
        .with_pattern("(?i)sha1|sha-1".to_string())
    }

    /// Insecure Random Number Generation
    fn insecure_random() -> Rule {
        Rule::new(
            "SEC-014".to_string(),
            "Insecure Random Number Generator".to_string(),
            "Using non-cryptographic random number generators for security purposes is unsafe. Use crypto-secure RNG.".to_string(),
            Severity::Medium,
            RuleCategory::Cryptography,
            vec!["javascript".to_string(), "typescript".to_string(), "python".to_string()],
        )
        .with_cwe("CWE-338".to_string())
        .with_owasp("A02:2021 - Cryptographic Failures".to_string())
        .with_pattern("Math\\.random|rand\\(\\)|random\\.random".to_string())
    }

    /// Hardcoded AWS Key
    fn hardcoded_aws_key() -> Rule {
        Rule::new(
            "SEC-015".to_string(),
            "Hardcoded AWS Access Key".to_string(),
            "AWS access keys should not be hardcoded. Use IAM roles or environment variables.".to_string(),
            Severity::Critical,
            RuleCategory::HardcodedSecrets,
            vec!["rust".to_string(), "javascript".to_string(), "python".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-798".to_string())
        .with_pattern("(?i)aws[_-]?(access[_-]?key|secret[_-]?key)\\s*[:=]\\s*['\"][A-Z0-9]{20}['\"]".to_string())
    }

    /// Hardcoded Database URL
    fn hardcoded_database_url() -> Rule {
        Rule::new(
            "SEC-016".to_string(),
            "Hardcoded Database URL".to_string(),
            "Database URLs with credentials should not be hardcoded. Use environment variables.".to_string(),
            Severity::Critical,
            RuleCategory::HardcodedSecrets,
            vec!["rust".to_string(), "javascript".to_string(), "python".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-798".to_string())
        .with_pattern("(?i)(database|db)_?url\\s*[:=]\\s*['\"][a-z]+://[^'\"]+['\"]".to_string())
    }

    /// Unsafe Deserialization
    fn unsafe_deserialization() -> Rule {
        Rule::new(
            "SEC-017".to_string(),
            "Unsafe Deserialization".to_string(),
            "Deserializing untrusted data can lead to remote code execution. Validate and sanitize input.".to_string(),
            Severity::Critical,
            RuleCategory::Deserialization,
            vec!["rust".to_string(), "javascript".to_string(), "python".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-502".to_string())
        .with_owasp("A08:2021 - Software and Data Integrity Failures".to_string())
        .with_pattern("(deserialize|unpickle|eval\\(|JSON\\.parse)".to_string())
    }

    /// Sensitive Data Logging
    fn sensitive_data_logging() -> Rule {
        Rule::new(
            "SEC-018".to_string(),
            "Sensitive Data in Logs".to_string(),
            "Logging sensitive data (passwords, tokens, keys) is a security risk. Redact or avoid logging sensitive data.".to_string(),
            Severity::Medium,
            RuleCategory::DataExposure,
            vec!["rust".to_string(), "javascript".to_string(), "python".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-532".to_string())
        .with_owasp("A09:2021 - Security Logging and Monitoring Failures".to_string())
        .with_pattern("(?i)(log|print|console\\.log|println!).*(password|token|key|secret)".to_string())
    }

    /// Hardcoded Auth Token
    fn hardcoded_auth_token() -> Rule {
        Rule::new(
            "SEC-019".to_string(),
            "Hardcoded Bearer Token".to_string(),
            "Bearer tokens should not be hardcoded. Use secure token storage.".to_string(),
            Severity::Critical,
            RuleCategory::HardcodedSecrets,
            vec!["rust".to_string(), "javascript".to_string(), "python".to_string(), "go".to_string()],
        )
        .with_cwe("CWE-798".to_string())
        .with_pattern("(?i)bearer\\s+['\"]?[a-zA-Z0-9]{20,}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_rules() {
        let rules = RuleLibrary::get_all_rules();
        assert!(!rules.is_empty());
        assert!(rules.len() > 10);
    }

    #[test]
    fn test_rule_ids_are_unique() {
        let rules = RuleLibrary::get_all_rules();
        let ids: Vec<&str> = rules.iter().map(|r| r.id.as_str()).collect();
        let unique_ids: std::collections::HashSet<_> = ids.iter().cloned().collect();
        assert_eq!(ids.len(), unique_ids.len());
    }

    #[test]
    fn test_rules_have_required_fields() {
        let rules = RuleLibrary::get_all_rules();
        for rule in rules {
            assert!(!rule.id.is_empty());
            assert!(!rule.name.is_empty());
            assert!(!rule.description.is_empty());
            assert!(!rule.languages.is_empty());
        }
    }
}
