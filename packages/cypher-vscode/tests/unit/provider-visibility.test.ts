import { describe, expect, it } from "bun:test"

import {
  disabledProviderOptions,
  providersWithCypherFallback,
  visibleConnectedIds,
} from "../../webview-ui/src/components/settings/provider-visibility"

describe("visibleConnectedIds", () => {
  it("hides Cypher from the connected list when auth is missing", () => {
    const ids = visibleConnectedIds(["cypher", "openrouter"], { openrouter: "api" })

    expect(ids).toEqual(["openrouter"])
  })

  it("keeps Cypher in the connected list when auth exists", () => {
    const ids = visibleConnectedIds(["cypher", "openrouter"], { cypher: "oauth", openrouter: "api" })

    expect(ids).toEqual(["cypher", "openrouter"])
  })

  it("leaves non-Cypher providers untouched", () => {
    const ids = visibleConnectedIds(["anthropic"], {})

    expect(ids).toEqual(["anthropic"])
  })
})

describe("disabledProviderOptions", () => {
  it("includes Cypher and excludes already disabled providers", () => {
    const options = disabledProviderOptions(
      {
        cypher: { id: "cypher", name: "Cypher Gateway", env: [], models: {} },
        openai: { id: "openai", name: "OpenAI", env: [], models: {} },
        anthropic: { id: "anthropic", name: "Anthropic", env: [], models: {} },
      },
      ["openai"],
    )

    expect(options).toEqual([
      { value: "anthropic", label: "Anthropic" },
      { value: "cypher", label: "Cypher Gateway" },
    ])
  })

  it("sorts options by provider name", () => {
    const options = disabledProviderOptions(
      {
        zed: { id: "zed", name: "Zed", env: [], models: {} },
        alpha: { id: "alpha", name: "Alpha", env: [], models: {} },
      },
      [],
    )

    expect(options).toEqual([
      { value: "alpha", label: "Alpha" },
      { value: "zed", label: "Zed" },
    ])
  })
})

describe("providersWithCypherFallback", () => {
  it("adds Cypher when backend providers omit it", () => {
    const providers = providersWithCypherFallback({
      anthropic: { id: "anthropic", name: "Anthropic", env: [], models: {} },
    })

    expect(providers.cypher?.name).toBe("Cypher Gateway")
    expect(providers.anthropic?.name).toBe("Anthropic")
  })

  it("keeps the backend Cypher provider when present", () => {
    const providers = providersWithCypherFallback({
      cypher: { id: "cypher", name: "Custom Cypher Name", env: [], models: {} },
    })

    expect(providers.cypher?.name).toBe("Custom Cypher Name")
  })
})
