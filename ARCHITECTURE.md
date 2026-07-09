# Cypher CLI Architecture

## Overview

Cypher CLI is a modular, production-grade cybersecurity auditing tool built with Rust. The architecture follows Clean Architecture principles with clear separation of concerns.

## Core Components

### 1. CLI Layer (`cli.rs`)

**Responsibility**: Command-line interface and argument parsing

**Key Features**:
- Uses `clap` for declarative argument parsing
- Supports multiple subcommands (scan, init, list-rules, validate, report, plugin)
- Global options for verbosity, configuration, and output formatting
- Type-safe command definitions

**Structure**:
```rust
pub struct Cli {
    pub verbose: u8,
    pub config: Option<PathBuf>,
    pub no_color: bool,
    pub command: Commands,
}

pub enum Commands {
    Scan { /* ... */ },
    Init { /* ... */ },
    ListRules { /* ... */ },
    Validate { /* ... */ },
    Report { /* ... */ },
    Plugin { /* ... */ },
}
```

### 2. Configuration System (`config.rs`)

**Responsibility**: Configuration management and validation

**Key Features**:
- Multi-format support (TOML, YAML, JSON)
- Default configuration with sensible defaults
- Configuration validation
- Environment variable support via clap

**Structure**:
```rust
pub struct Config {
    pub general: GeneralConfig,
    pub scanner: ScannerConfig,
    pub rules: RulesConfig,
    pub reporting: ReportingConfig,
    pub plugins: PluginsConfig,
}
```

**Configuration Loading**:
1. Check explicit `--config` flag
2. Search default locations (cypher.toml, cypher.yaml, etc.)
3. Fall back to default configuration
4. Validate loaded configuration

### 3. Error Handling (`error.rs`)

**Responsibility**: Centralized error types and handling

**Key Features**:
- Comprehensive error enum covering all error scenarios
- `thiserror` for automatic error display implementations
- `anyhow` for ergonomic error propagation in application code

**Error Categories**:
- Configuration errors
- I/O errors
- Parse errors
- Rule errors
- Plugin errors
- Report errors
- Scanner errors
- AST parsing errors

### 4. Rule Engine (`rules/`)

**Responsibility**: Security rule management and execution

#### Components:

**Rule Definition (`rule.rs`)**:
```rust
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub category: RuleCategory,
    pub cwe: Option<String>,
    pub owasp: Option<String>,
    pub languages: Vec<String>,
    pub enabled: bool,
    pub pattern: Option<String>,
    pub metadata: serde_json::Value,
}
```

**Severity Levels (`severity.rs`)**:
- Low
- Medium
- High
- Critical

**Rule Categories (`category.rs`)**:
- SQL Injection
- XSS
- Command Injection
- Path Traversal
- Cryptography
- Authentication
- Data Exposure
- Dependency
- Configuration
- Error Handling
- Logging
- Input Validation
- API Security
- Hardcoded Secrets
- Deserialization
- Race Condition

**Rule Engine (`engine.rs`)**:
```rust
pub struct RuleEngine {
    rules: HashMap<String, Rule>,
    rules_by_category: HashMap<RuleCategory, Vec<String>>,
    rules_by_language: HashMap<String, Vec<String>>,
    severity_threshold: Severity,
}
```

**Capabilities**:
- Rule registration and management
- Category-based filtering
- Language-based filtering
- Severity threshold filtering
- Rule execution with pattern matching
- AST-based rule evaluation (planned)

**Rule Library (`library.rs`)**:
- 19 built-in security rules
- CWE and OWASP references
- Multi-language support
- Regex-based pattern matching

### 5. AST Parser (`parser/`)

**Responsibility**: Code parsing and AST generation

#### Components:

**Language Support (`language.rs`)**:
```rust
pub enum Language {
    Rust, JavaScript, TypeScript, Python, Go,
    Java, C, Cpp, Csharp, Php, Ruby, Swift, Kotlin, Scala,
}
```

**AST Node Definition (`ast.rs`)**:
```rust
pub struct AstNode {
    pub node_type: AstNodeType,
    pub text: String,
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub children: Vec<AstNode>,
    pub metadata: HashMap<String, String>,
}
```

**AST Node Types**:
- Source file
- Function definitions and calls
- Variable declarations and assignments
- Control flow (if, while, for)
- Literals (string, number, boolean)
- Security-relevant nodes (SQL queries, shell commands, etc.)

**Parser Implementation (`parser.rs`)**:
```rust
pub struct Parser {
    language: Language,
}
```

**Capabilities**:
- tree-sitter integration for accurate parsing
- Multi-language support
- AST traversal and querying
- Security-relevant node detection
- Position tracking for precise reporting

### 6. Reporting Engine (`reporter/`)

**Responsibility**: Report generation in multiple formats

**Planned Formats**:
- Text (console output)
- JSON (machine-readable)
- SARIF (standardized security results)
- HTML (human-readable reports)

**Structure**:
```rust
pub trait Reporter {
    fn generate(&self, results: &[RuleResult]) -> Result<String>;
}
```

### 7. Plugin System (`plugins/`)

**Responsibility**: Extensible architecture for custom rules

**Planned Features**:
- Dynamic plugin loading via `libloading`
- Plugin API for custom rule definitions
- Plugin discovery and management
- Sandboxed execution

## Data Flow

### Scan Operation Flow

```
1. CLI Parsing
   ↓
2. Configuration Loading
   ↓
3. File Discovery
   ↓
4. Language Detection
   ↓
5. AST Parsing
   ↓
6. Rule Selection
   ↓
7. Rule Execution
   ↓
8. Result Aggregation
   ↓
9. Report Generation
   ↓
10. Output
```

### Rule Execution Flow

```
1. Load Rule
   ↓
2. Check Language Compatibility
   ↓
3. Check Severity Threshold
   ↓
4. Parse Source Code (AST)
   ↓
5. Execute Pattern Matching
   ↓
6. Execute AST Analysis (if applicable)
   ↓
7. Collect Matches
   ↓
8. Generate RuleResult
   ↓
9. Return Results
```

## Design Principles

### 1. Modularity

Each component has a single, well-defined responsibility. Modules are loosely coupled through well-defined interfaces.

### 2. Extensibility

- Plugin system for custom rules
- Reporter trait for custom output formats
- Language enum for adding new language support

### 3. Performance

- Parallel file scanning
- Incremental parsing
- Efficient rule matching
- Configurable thread pool

### 4. Security

- No hardcoded secrets in code
- Secure-by-default configuration
- Sandboxed plugin execution (planned)
- Input validation

### 5. Testability

- Dependency injection for testing
- Mockable interfaces
- Comprehensive test coverage
- Integration tests for end-to-end scenarios

## Technology Stack

### Core Dependencies

- **clap 4.5**: CLI argument parsing
- **tokio 1.35**: Async runtime
- **serde 1.0**: Serialization
- **tree-sitter 0.22**: AST parsing
- **regex 1.12**: Pattern matching
- **tracing 0.1**: Structured logging

### Language Support

- **tree-sitter-rust 0.21**: Rust parsing
- **tree-sitter-javascript 0.21**: JavaScript/TypeScript parsing
- **tree-sitter-python 0.21**: Python parsing
- **tree-sitter-go 0.21**: Go parsing

### Testing

- **tempfile 3.10**: Temporary file management
- **assert_cmd 2.0**: CLI testing
- **predicates 3.1**: Assertion predicates

## Future Enhancements

### Short Term

1. **Reporting Engine**: Implement JSON, SARIF, and HTML reporters
2. **Scanner Module**: Implement file discovery and scanning logic
3. **Framework Detection**: Detect web frameworks (React, Django, etc.)

### Medium Term

1. **Plugin System**: Complete plugin architecture
2. **AST-based Rules**: Enhance rules with AST analysis
3. **Incremental Scanning**: Only scan changed files
4. **Cache Layer**: Cache AST results for performance

### Long Term

1. **Remote Analysis**: Cloud-based scanning
2. **Machine Learning**: ML-powered vulnerability detection
3. **IDE Integration**: VS Code, JetBrains extensions
4. **Custom Rule Builder**: Web-based rule creation interface

## Security Considerations

### Input Validation

- All file paths are validated before access
- Configuration files are validated before use
- User input is sanitized before pattern matching

### Resource Limits

- Maximum file size limits
- Maximum recursion depth for AST traversal
- Timeout for rule execution

### Data Privacy

- No telemetry by default
- Optional anonymous usage statistics
- Local-only analysis by default

## Performance Optimization

### Parallel Processing

- File scanning is parallelized using tokio
- Rule execution can be parallelized
- Configurable thread pool size

### Caching

- AST results can be cached
- Rule patterns are pre-compiled
- File system operations are batched

### Memory Management

- Streaming file reading for large files
- Limit on concurrent file handles
- Efficient data structures for rule storage

## Testing Strategy

### Unit Tests

- Test individual components in isolation
- Mock external dependencies
- Cover edge cases and error conditions

### Integration Tests

- Test end-to-end workflows
- Test CLI commands
- Test configuration loading and validation

### Regression Tests

- Test known vulnerabilities are detected
- Test false positives are minimized
- Test performance benchmarks

## Documentation

### Code Documentation

- All public APIs are documented with rustdoc
- Complex algorithms have inline comments
- Architecture decisions are documented in ARCHITECTURE.md

### User Documentation

- README.md for quick start
- CLI reference for all commands
- Configuration reference for all options
- Rule reference for all security rules

### Developer Documentation

- ARCHITECTURE.md for system design
- CONTRIBUTING.md for contribution guidelines
- CODE_OF_CONDUCT.md for community standards
