# Cypher CLI

Production-grade cybersecurity auditing CLI for developers.

## Overview

Cypher CLI is a static security analysis tool designed to help developers identify security vulnerabilities in their codebases. It supports multiple programming languages and provides comprehensive security rule coverage.

## Features

- **Multi-Language Support**: Rust, JavaScript, TypeScript, Python, Go, and more
- **AST-Based Analysis**: Uses tree-sitter for accurate code parsing
- **Comprehensive Security Rules**: 19+ built-in security rules covering:
  - Hardcoded secrets (API keys, passwords, tokens)
  - SQL injection vulnerabilities
  - Command injection
  - Path traversal
  - Cross-site scripting (XSS)
  - Weak cryptography
  - Insecure deserialization
  - Sensitive data exposure
- **Flexible Configuration**: TOML, YAML, and JSON support
- **Multiple Output Formats**: Text, JSON, SARIF, HTML
- **Plugin System**: Extensible architecture for custom rules
- **CI/CD Integration**: Easy integration into development pipelines

## Installation

Install Cypher CLI directly on your machine. No Rust, Cargo, or compiler dependencies are required.

### 1. Using package managers

#### npm (All Platforms)
```bash
npm install -g @cypher/cli
```

#### Homebrew (macOS & Linux)
```bash
brew install Ayan-Flash/tap/cypher
```

---

### 2. Using Shell Installers (Fast Setup)

#### On macOS / Linux (Curl):
```bash
curl -fsSL https://raw.githubusercontent.com/Ayan-Flash/Cypher/main/scripts/install.sh | bash
```

#### On Windows (PowerShell):
```powershell
iwr -useb https://raw.githubusercontent.com/Ayan-Flash/Cypher/main/scripts/install.ps1 | iex
```
*Note: The Windows installer automatically configures your User PATH environment variable.*

Once installed, verify the installation by checking the version:
```bash
cypher --version
```

---

### 3. Manual Installation from Source
If you are a contributor or prefer compiling from source, you will need the Rust toolchain:
```bash
# Clone the repository
git clone https://github.com/Ayan-Flash/Cypher.git
cd Cypher
```

# Build the project
cargo build --release
```

### Using Cargo

```bash
cargo install --path .
```

## Quick Start

### Initialize Configuration

```bash
cypher init
```

This creates a `cypher.toml` configuration file in your current directory.

### Scan a Project

```bash
# Scan current directory
cypher scan

# Scan specific directory
cypher scan ./src

# Scan with severity threshold
cypher scan ./src --severity high

# Scan with specific output format
cypher scan ./src --output json --file report.json
```

### List Available Rules

```bash
# List all rules
cypher list-rules

# Filter by severity
cypher list-rules --severity critical

# Filter by category
cypher list-rules --category hardcoded_secrets
```

### Validate Configuration

```bash
cypher validate cypher.toml
```

## Configuration

Cypher CLI uses a configuration file to customize behavior. The default locations are:

- `cypher.toml` (current directory)
- `cypher.yaml` / `cypher.yml`
- `cypher.json`
- `.cypher/config.toml`

### Example Configuration

```toml
[general]
verbose = false
color = true
max_threads = 4

[scanner]
exclude_paths = ["node_modules", "target", "vendor", ".git"]
exclude_patterns = ["*.min.js", "*.min.css"]
max_file_size = 10485760  # 10MB
follow_symlinks = false

[rules]
severity_threshold = "low"
enabled_rules = []
disabled_rules = []
enabled_categories = []

[reporting]
format = "text"
include_snippets = true
max_issues = 1000

[plugins]
directory = ".cypher/plugins"
auto_load = true
```

## Security Rules

### Hardcoded Secrets

- **SEC-001**: Hardcoded API Key
- **SEC-002**: Hardcoded Password
- **SEC-003**: Hardcoded Authentication Token
- **SEC-004**: Hardcoded Private Key
- **SEC-015**: Hardcoded AWS Access Key
- **SEC-016**: Hardcoded Database URL
- **SEC-019**: Hardcoded Bearer Token

### Injection Vulnerabilities

- **SEC-005**: SQL Injection via String Concatenation
- **SEC-006**: SQL Injection via String Format
- **SEC-007**: Unsafe Command Execution
- **SEC-008**: Shell Command with User Input
- **SEC-009**: Path Traversal Vulnerability

### Cross-Site Scripting

- **SEC-010**: XSS via innerHTML Assignment
- **SEC-011**: XSS via document.write

### Cryptography

- **SEC-012**: Weak Hash Algorithm (MD5)
- **SEC-013**: Weak Hash Algorithm (SHA1)
- **SEC-014**: Insecure Random Number Generator

### Other Vulnerabilities

- **SEC-017**: Unsafe Deserialization
- **SEC-018**: Sensitive Data in Logs

## CLI Reference

### Global Options

- `-v, --verbose`: Increase verbosity level (can be used multiple times)
- `-c, --config <CONFIG>`: Path to configuration file
- `--no-color`: Disable colored output
- `-h, --help`: Print help
- `-V, --version`: Print version

### Commands

#### `scan`

Scan a directory or file for security issues.

```bash
cypher scan [OPTIONS] [PATH]
```

**Options:**
- `-o, --output <FORMAT>`: Output format (text, json, sarif, html) [default: text]
- `-f, --file <FILE>`: Output file path
- `-s, --severity <LEVEL>`: Severity threshold (low, medium, high, critical) [default: low]
- `-r, --rules <RULES>`: Specific rules to run (comma-separated)
- `-e, --exclude-rules <RULES>`: Rules to exclude (comma-separated)
- `-m, --max-issues <NUM>`: Maximum number of issues to report
- `--fail-on-issues`: Exit with non-zero status if issues found

#### `init`

Initialize Cypher configuration.

```bash
cypher init [OPTIONS]
```

**Options:**
- `--force`: Force overwrite existing configuration

#### `list-rules`

List available security rules.

```bash
cypher list-rules [OPTIONS]
```

**Options:**
- `-s, --severity <LEVEL>`: Filter by severity
- `--category <CATEGORY>`: Filter by category

#### `validate`

Validate configuration file.

```bash
cypher validate <CONFIG>
```

#### `report`

Generate a security report.

```bash
cypher report [OPTIONS] <PATH>
```

**Options:**
- `-f, --format <FORMAT>`: Report format (json, sarif, html, pdf) [default: json]
- `-o, --output <FILE>`: Output file path

#### `plugin`

Manage plugins.

```bash
cypher plugin <COMMAND>
```

**Subcommands:**
- `list`: List installed plugins
- `install <PLUGIN>`: Install a plugin
- `remove <NAME>`: Remove a plugin
- `update`: Update all plugins

#### `ask`

Ask a security-related question to the Cypher AI Assistant. If the prompt is omitted, it starts a premium interactive shell (REPL) session similar to `claude-code`. If no key has been configured, the assistant will automatically boot an interactive configuration wizard to prompt for your key, auto-detect the provider, and let you choose from supported models.

```bash
cypher ask [OPTIONS] [PROMPT]
```

**Options:**
- `-a, --api-key <API_KEY>`: AI API Key (can also be set via the `GEMINI_API_KEY` environment variable or the `[ai]` section in `settings.json`).


## Development

### Building

```bash
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests
cargo test --test integration_test
```

### Project Structure

```
cypher-cli/
├── src/
│   ├── cli.rs              # CLI argument parsing
│   ├── config.rs           # Configuration management
│   ├── error.rs            # Error types
│   ├── main.rs             # Application entry point
│   ├── parser/             # AST parsing
│   │   ├── mod.rs
│   │   ├── ast.rs
│   │   ├── language.rs
│   │   └── parser.rs
│   ├── reporter/           # Report generation
│   │   ├── mod.rs
│   │   ├── json.rs
│   │   ├── sarif.rs
│   │   ├── text.rs
│   │   └── html.rs
│   ├── rules/              # Security rules
│   │   ├── mod.rs
│   │   ├── engine.rs
│   │   ├── rule.rs
│   │   ├── severity.rs
│   │   ├── category.rs
│   │   └── library.rs
│   └── plugins/            # Plugin system
├── tests/
│   └── integration_test.rs
├── Cargo.toml
└── README.md
```

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting pull requests.

## License

MIT License - see LICENSE file for details.

## Security

For security vulnerabilities, please email security@cypher-security.com instead of using the issue tracker.

## Acknowledgments

- [tree-sitter](https://tree-sitter.github.io/) - Parser generator tool
- [clap](https://github.com/clap-rs/clap) - Command line argument parser
- OWASP Top 10 2021 - Security rule references
