---
title: "General"
description: "General questions about Cypher CLI"
---

# General

This section contains general questions about Cypher CLI.

## How does Cypher CLI work?

Cypher CLI uses large language models (LLMs) to understand your requests and translate them into actions. It can:

- Read, write, and delete files in your project.
- Execute commands in your VS Code terminal.
- Perform web browsing (if enabled).
- Use external tools via the Model Context Protocol (MCP).

You interact with Cypher CLI through a chat interface, where you provide instructions and review/approve its proposed actions, or you can use the inline autocomplete feature which helps you as you type.

## Is Cypher CLI free to use?

The Cypher CLI extension itself is free and open-source. In order for Cypher CLI to be useful, you need an AI model to respond to your queries. Models are hosted by providers and most charge for access.

There are some [models](https://cypher.ai/leaderboard#all-models) available for free. The set of free models is constantly changing based on provider pricing decisions.

You can also use Cypher CLI with a [local model](/docs/automate/extending/local-models) or ["Bring Your Own API Key"](/docs/getting-started/byok).
