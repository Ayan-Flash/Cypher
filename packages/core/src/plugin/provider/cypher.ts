import { createCypher, CYPHER_OPENROUTER_BASE } from "@cypher/cypher-gateway" // cypher_change
import { Effect } from "effect"
import { PluginV2 } from "../../plugin"
import { ProviderV2 } from "../../provider" // cypher_change

const id = ProviderV2.ID.make("cypher") // cypher_change

export const CypherPlugin = PluginV2.define({
  id: PluginV2.ID.make("cypher"),
  effect: Effect.gen(function* () {
    return {
      "catalog.transform": Effect.fn(function* (evt) {
        for (const item of evt.provider.list()) {
          if (item.provider.id !== id) continue // cypher_change
          evt.provider.update(item.provider.id, (provider) => {
            // cypher_change start
            const options = provider.options.aisdk.provider
            const token = options.cypherToken ?? options.apiKey ?? process.env.CYPHER_API_KEY ?? process.env.CYPHER_API_KEY
            const org = process.env.CYPHER_ORG_ID ?? process.env.CYPHER_ORG_ID ?? options.cypherOrganizationId

            provider.endpoint = {
              type: "aisdk",
              package: "@cypher/cypher-gateway",
              url: CYPHER_OPENROUTER_BASE,
            }
            // cypher_change end
            provider.options.headers["HTTP-Referer"] = "https://cypher.ai/"
            // cypher_change start
            provider.options.headers["X-Title"] = "Cypher CLI"
            options.cypherToken = token ?? "anonymous"
            if (org) options.cypherOrganizationId = org
            if (!provider.enabled) provider.enabled = { via: "custom", data: { anonymous: true } }
            // cypher_change end
          })
        }
      }),
      // cypher_change start
      "aisdk.sdk": Effect.fn(function* (evt) {
        if (evt.model.providerID !== id) return
        evt.sdk = createCypher(evt.options)
      }),
      // cypher_change end
    }
  }),
})
