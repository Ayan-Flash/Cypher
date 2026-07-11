# @cypher/cypher-gateway

Unified Cypher Gateway package for OpenCode providing authentication, AI provider integration, and API access.

## Features

- **Authentication**: Device authorization flow for Cypher Gateway
- **AI Provider**: OpenRouter-based provider with Cypher Gateway integration
- **API Integration**: Profile, balance, and model management
- **TUI Helpers**: Utilities for terminal UI components

## Installation

```bash
bun add @cypher/cypher-gateway
```

## Usage

### Plugin Registration

```typescript
import { CypherAuthPlugin } from "@cypher/cypher-gateway"

// Register with OpenCode
const plugins = [CypherAuthPlugin]
```

### Provider Usage

```typescript
import { createCypher } from "@cypher/cypher-gateway"

const provider = createCypher({
  cypherToken: process.env.CYPHER_API_KEY,
  cypherOrganizationId: "org-123",
})

const model = provider.languageModel("anthropic/claude-sonnet-4")
```

### API Access

```typescript
import { fetchProfile, fetchBalance } from "@cypher/cypher-gateway"

const profile = await fetchProfile(token)
const balance = await fetchBalance(token)
```

## License

MIT
