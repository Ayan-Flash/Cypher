import { describe, expect } from "bun:test"
import { Effect } from "effect"
import { Catalog } from "@opencode-ai/core/catalog"
import { PluginV2 } from "@opencode-ai/core/plugin"
import { ProviderPlugins } from "@opencode-ai/core/plugin/provider"
import { CypherPlugin } from "@opencode-ai/core/plugin/provider/cypher"
import { ProviderV2 } from "@opencode-ai/core/provider"
import { expectPluginRegistered, it, model, provider, withEnv } from "./provider-helper" // cypher_change

describe("CypherPlugin", () => {
  it.effect("is registered so legacy referer headers can be applied", () =>
    Effect.sync(() =>
      expectPluginRegistered(
        ProviderPlugins.map((item) => item.id),
        "cypher",
      ),
    ),
  )

  it.effect("applies legacy referer headers only to cypher", () =>
    Effect.gen(function* () {
      const plugin = yield* PluginV2.Service
      const catalog = yield* Catalog.Service
      yield* plugin.add(CypherPlugin)
      const transform = yield* catalog.transform()
      yield* transform((catalog) => {
        const cypher = provider("cypher", {
          endpoint: { type: "aisdk", package: "@ai-sdk/openai-compatible", url: "https://api.cypher.ai/api/gateway" },
          options: { headers: { Existing: "value" }, body: {}, aisdk: { provider: {}, request: {} } },
        })
        catalog.provider.update(cypher.id, (draft) => {
          draft.endpoint = cypher.endpoint
          draft.options = cypher.options
        })
        catalog.provider.update(provider("openrouter").id, () => {})
      })
      expect((yield* catalog.provider.get(ProviderV2.ID.make("cypher"))).options.headers).toEqual({
        Existing: "value",
        "HTTP-Referer": "https://cypher.ai/",
        "X-Title": "Cypher CLI", // cypher_change
      })
      expect((yield* catalog.provider.get(ProviderV2.ID.openrouter)).options.headers).toEqual({})
    }),
  )

  it.effect("uses the exact legacy Cypher header casing and set", () =>
    Effect.gen(function* () {
      const plugin = yield* PluginV2.Service
      const catalog = yield* Catalog.Service
      yield* plugin.add(CypherPlugin)
      const transform = yield* catalog.transform()
      yield* transform((catalog) => {
        const item = provider("cypher", {
          endpoint: { type: "aisdk", package: "@ai-sdk/openai-compatible", url: "https://api.cypher.ai/api/gateway" },
        })
        catalog.provider.update(item.id, (draft) => {
          draft.endpoint = item.endpoint
        })
      })

      const result = yield* catalog.provider.get(ProviderV2.ID.make("cypher"))
      expect(result.options.headers).toEqual({
        "HTTP-Referer": "https://cypher.ai/",
        "X-Title": "Cypher CLI", // cypher_change
      })
      expect(result.options.headers).not.toHaveProperty("http-referer")
      expect(result.options.headers).not.toHaveProperty("x-title")
      expect(result.options.headers).not.toHaveProperty("X-Source")
    }),
  )

  it.effect("uses the legacy provider-id guard instead of endpoint package matching", () =>
    Effect.gen(function* () {
      const plugin = yield* PluginV2.Service
      const catalog = yield* Catalog.Service
      yield* plugin.add(CypherPlugin)
      const transform = yield* catalog.transform()
      yield* transform((catalog) => {
        const cypher = provider("cypher", {
          endpoint: { type: "aisdk", package: "@ai-sdk/openai-compatible", url: "https://api.cypher.ai/api/gateway" },
        })
        catalog.provider.update(cypher.id, (draft) => {
          draft.endpoint = cypher.endpoint
        })
        const custom = provider("custom-cypher", {
          endpoint: { type: "aisdk", package: "cypher" },
        })
        catalog.provider.update(custom.id, (draft) => {
          draft.endpoint = custom.endpoint
        })
      })

      expect((yield* catalog.provider.get(ProviderV2.ID.make("cypher"))).options.headers).toEqual({
        "HTTP-Referer": "https://cypher.ai/",
        "X-Title": "Cypher CLI", // cypher_change
      })
      expect((yield* catalog.provider.get(ProviderV2.ID.make("custom-cypher"))).options.headers).toEqual({})
    }),
  )

  // cypher_change start
  it.effect("routes the Cypher catalog through the Cypher Gateway SDK", () =>
    withEnv({ CYPHER_API_KEY: undefined, CYPHER_ORG_ID: undefined }, () =>
      Effect.gen(function* () {
        const plugin = yield* PluginV2.Service
        const catalog = yield* Catalog.Service
        yield* plugin.add(CypherPlugin)
        const transform = yield* catalog.transform()
        yield* transform((catalog) => {
          const item = provider("cypher", {
            endpoint: { type: "aisdk", package: "@ai-sdk/openai-compatible", url: "https://api.cypher.ai/api/gateway" },
            options: {
              headers: {},
              body: {},
              aisdk: { provider: { apiKey: "stored-token" }, request: {} },
            },
          })
          catalog.provider.update(item.id, (draft) => {
            draft.endpoint = item.endpoint
            draft.options = item.options
          })
        })
        const updated = yield* catalog.provider.get(ProviderV2.ID.make("cypher"))

        expect(updated.endpoint).toEqual({
          type: "aisdk",
          package: "@cypher/cypher-gateway",
          url: "https://api.cypher.ai/api/openrouter",
        })
        expect(updated.options.aisdk.provider.cypherToken).toBe("stored-token")

        const result = yield* plugin.trigger(
          "aisdk.sdk",
          {
            model: model("cypher", "cypher-auto/free"),
            package: "@cypher/cypher-gateway",
            options: updated.options.aisdk.provider,
          },
          {},
        )
        expect(result.sdk).toBeDefined()
        expect(typeof result.sdk.languageModel).toBe("function")
        expect(typeof result.sdk.anthropic).toBe("function")
      }),
    ),
  )

  it.effect("keeps authenticated credentials ahead of inherited environment keys", () =>
    withEnv({ CYPHER_API_KEY: "environment-token", CYPHER_ORG_ID: "environment-org" }, () =>
      Effect.gen(function* () {
        const plugin = yield* PluginV2.Service
        const catalog = yield* Catalog.Service
        yield* plugin.add(CypherPlugin)
        const transform = yield* catalog.transform()
        yield* transform((catalog) => {
          const item = provider("cypher", {
            enabled: { via: "account", service: "cypher" },
            options: {
              headers: {},
              body: {},
              aisdk: {
                provider: { apiKey: "authenticated-token", cypherOrganizationId: "authenticated-org" },
                request: {},
              },
            },
          })
          catalog.provider.update(item.id, (draft) => {
            draft.enabled = item.enabled
            draft.options = item.options
          })
        })
        const result = yield* catalog.provider.get(ProviderV2.ID.make("cypher"))

        expect(result.enabled).toEqual({ via: "account", service: "cypher" })
        expect(result.options.aisdk.provider.cypherToken).toBe("authenticated-token")
        expect(result.options.aisdk.provider.cypherOrganizationId).toBe("environment-org")
      }),
    ),
  )

  it.effect("keeps anonymous Cypher models available without credentials", () =>
    withEnv({ CYPHER_API_KEY: undefined, CYPHER_ORG_ID: undefined }, () =>
      Effect.gen(function* () {
        const plugin = yield* PluginV2.Service
        const catalog = yield* Catalog.Service
        yield* plugin.add(CypherPlugin)
        const transform = yield* catalog.transform()
        yield* transform((catalog) => catalog.provider.update(ProviderV2.ID.make("cypher"), () => {}))
        const result = yield* catalog.provider.get(ProviderV2.ID.make("cypher"))

        expect(result.enabled).toEqual({ via: "custom", data: { anonymous: true } })
        expect(result.options.aisdk.provider.cypherToken).toBe("anonymous")
      }),
    ),
  )
  // cypher_change end
})
