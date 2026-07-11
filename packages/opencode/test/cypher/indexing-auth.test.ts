import { describe, expect, test } from "bun:test"
import {
  hasCypherIndexingAuth,
  resolveCypherIndexingAuth,
  shouldDefaultIndexingToCypher,
} from "../../src/cypher/indexing-auth"

describe("Cypher indexing auth resolution", () => {
  test("detects auth from explicit indexing Cypher config", () => {
    const auth = resolveCypherIndexingAuth({
      config: { indexing: { cypher: { apiKey: "idx-token", baseUrl: "https://idx.test", organizationId: "org_idx" } } },
    })

    expect(auth).toEqual({ apiKey: "idx-token", baseUrl: "https://idx.test", organizationId: "org_idx" })
    expect(hasCypherIndexingAuth({ config: { indexing: { cypher: { apiKey: "idx-token" } } } })).toBe(true)
  })

  test("detects auth from provider config, provider state, auth storage, and env", () => {
    expect(
      resolveCypherIndexingAuth({ config: { provider: { cypher: { options: { apiKey: "cfg-token" } } } } }).apiKey,
    ).toBe("cfg-token")
    expect(resolveCypherIndexingAuth({ provider: { options: { cypherToken: "provider-token" } } }).apiKey).toBe(
      "provider-token",
    )
    expect(resolveCypherIndexingAuth({ auth: { type: "oauth", access: "oauth-token", accountId: "org_oauth" } })).toEqual(
      {
        apiKey: "oauth-token",
        organizationId: "org_oauth",
      },
    )
    expect(resolveCypherIndexingAuth({ env: { CYPHER_API_KEY: "env-token", CYPHER_ORG_ID: "org_env" } })).toEqual({
      apiKey: "env-token",
      organizationId: "org_env",
    })
  })

  test("defaults to Cypher only when no provider or other embedder config is present", () => {
    const auth = { apiKey: "cypher-token" }

    expect(shouldDefaultIndexingToCypher({}, auth)).toBe(true)
    expect(shouldDefaultIndexingToCypher({ provider: "openai" }, auth)).toBe(false)
    expect(shouldDefaultIndexingToCypher({ openai: { apiKey: "openai-key" } }, auth)).toBe(false)
    expect(shouldDefaultIndexingToCypher({ ollama: { baseUrl: "http://localhost:11434" } }, auth)).toBe(false)
  })
})
