# Cypher CLI CLI

The AI coding agent built for the terminal. Generate code from natural language, automate tasks, and run terminal commands -- powered by 500+ AI models.

![Cypher CLI showing code edits in a terminal](https://raw.githubusercontent.com/Cypher-Org/cypher/main/packages/cypher-docs/public/img/npm-package-readme/cypher-cli.png)

Cypher is the all-in-one agentic engineering platform. Build, ship, and iterate faster with the most popular open source coding agent.

[Website](https://cypher.ai) · [Install](https://cypher.ai/install) · [IDE](https://cypher.ai/landing/vs-code) · [CLI](https://cypher.ai/cli) · [Docs](https://cypher.ai/docs) · [Models](https://cypher.ai/leaderboard) · [Gateway](https://cypher.ai/gateway) · [Pricing](https://cypher.ai/pricing) · [Cypher Pass](https://cypher.ai/pricing/cypher-pass)

[500+ models](https://cypher.ai/leaderboard). One open source agent in [VS Code](https://cypher.ai/vscode-marketplace), [JetBrains](https://plugins.jetbrains.com/plugin/27133-cypher-cli), [CLI](https://www.npmjs.com/package/@cypher/cli), [Slack](https://cypher.ai/slack), and [Cloud](https://cypher.ai/cloud).

## Install

```bash
npm install -g @cypher/cli
```

Or run directly with npx:

```bash
npx --package @cypher/cli cypher
```

## Getting Started

Run `cypher` in any project directory to launch the interactive TUI:

```bash
cypher
```

Run a one-off task:

```bash
cypher run "add input validation to the signup form"
```

## Features

- **Code generation** -- describe what you want in natural language
- **Terminal commands** -- the agent can run shell commands on your behalf
- **500+ AI models** -- use models from OpenAI, Anthropic, Google, and more
- **MCP servers** -- extend agent capabilities with the Model Context Protocol
- **Multiple modes** -- Plan with Architect, code with Coder, debug with Debugger, or create your own
- **Sessions** -- resume previous conversations and export transcripts
- **API keys optional** -- bring your own keys or use Cypher credits

## Commands

| Command               | Description                |
| --------------------- | -------------------------- |
| `cypher`                | Launch interactive TUI     |
| `cypher run "<task>"`   | Run a one-off task         |
| `cypher auth`           | Manage authentication      |
| `cypher models`         | List available models      |
| `cypher mcp`            | Manage MCP servers         |
| `cypher session list`   | List sessions              |
| `cypher session delete` | Delete a session           |
| `cypher export`         | Export session transcripts |

Run `cypher --help` for the full list.

## Alternative Installation

### Homebrew (macOS/Linux)

```bash
brew install Cypher-Org/tap/cypher
```

### GitHub Releases

Download pre-built binaries from the [Releases page](https://github.com/Cypher-Org/cypher/releases).

## Documentation

- [Docs](https://cypher.ai/docs)
- [Getting Started](https://cypher.ai/docs/getting-started)

## Links

- [GitHub](https://github.com/Cypher-Org/cypher)
- [Discord](https://cypher.ai/discord)
- [VS Code Extension](https://cypher.ai/vscode-marketplace)
- [Website](https://cypher.ai)

## License

MIT
