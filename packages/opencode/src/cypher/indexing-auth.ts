import type { IndexingConfig } from "@cypher/cypher-indexing/config"

type Auth = unknown

type Env = {
  CYPHER_API_KEY?: string
  CYPHER_ORG_ID?: string
}

type Provider = {
  key?: unknown
  options?: Record<string, unknown>
}

export type CypherIndexingAuth = {
  apiKey?: string
  baseUrl?: string
  organizationId?: string
}

const providers = [
  "openai",
  "ollama",
  "openai-compatible",
  "gemini",
  "mistral",
  "vercel-ai-gateway",
  "bedrock",
  "openrouter",
  "voyage",
]

function record(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) return {}
  return value as Record<string, unknown>
}

function text(value: unknown): string | undefined {
  if (typeof value !== "string") return
  const trimmed = value.trim()
  return trimmed || undefined
}

function token(auth: Auth): string | undefined {
  const data = record(auth)
  if (data.type === "api") return text(data.key)
  if (data.type === "oauth") return text(data.access)
  return
}

function org(auth: Auth): string | undefined {
  const data = record(auth)
  if (data.type === "oauth") return text(data.accountId)
  return
}

function value(input: unknown): boolean {
  if (input === undefined || input === null) return false
  if (typeof input === "string") return input.trim().length > 0
  if (typeof input === "object") return Object.values(input).some(value)
  return true
}

function hasOtherProvider(indexing: unknown): boolean {
  const cfg = record(indexing)
  return providers.some((provider) => value(cfg[provider]))
}

export function resolveCypherIndexingAuth(input: {
  config?: unknown
  provider?: Provider
  auth?: Auth
  env?: Env
}): CypherIndexingAuth {
  const config = record(input.config)
  const options = record(record(config.provider).cypher)
  const provider = input.provider ?? record(input.provider)
  const providerOptions = record(provider.options)
  const providerConfig = record(options.options)
  const cypher = record(record(config.indexing).cypher)
  const env = input.env ?? process.env

  return {
    apiKey:
      text(cypher.apiKey) ??
      text(providerConfig.apiKey) ??
      token(input.auth) ??
      text(provider.key) ??
      text(providerOptions.cypherToken) ??
      text(env.CYPHER_API_KEY),
    baseUrl: text(cypher.baseUrl) ?? text(providerConfig.baseURL) ?? text(providerConfig.baseUrl),
    organizationId:
      text(cypher.organizationId) ??
      text(providerConfig.cypherOrganizationId) ??
      org(input.auth) ??
      text(providerOptions.cypherOrganizationId) ??
      text(env.CYPHER_ORG_ID),
  }
}

export function hasCypherIndexingAuth(input: Parameters<typeof resolveCypherIndexingAuth>[0]): boolean {
  return !!resolveCypherIndexingAuth(input).apiKey
}

export function shouldDefaultIndexingToCypher(indexing: unknown, auth: CypherIndexingAuth): boolean {
  const cfg = record(indexing)
  if (cfg.provider !== undefined || !auth.apiKey) return false
  return !hasOtherProvider(cfg)
}

export function indexingWithCypherDefault(
  indexing: IndexingConfig | undefined,
  auth: CypherIndexingAuth,
): IndexingConfig | undefined {
  if (!shouldDefaultIndexingToCypher(indexing, auth)) return indexing
  return { ...indexing, provider: "cypher" }
}
