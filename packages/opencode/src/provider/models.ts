// cypher_change - new file
import { Config } from "@/config/config"
import { Auth } from "@/auth"
import { ModelCache } from "./model-cache"
import * as Core from "@opencode-ai/core/models-dev"
import { Context, Effect, Layer } from "effect"
import { AI_SDK_PROVIDERS, CYPHER_OPENROUTER_BASE, PROMPTS } from "@cypher/cypher-gateway"
import { overlay } from "@/cypher/anaconda-desktop/provider"

export const Model = Core.Model
export type Model = Core.Model
export const Provider = Core.Provider
export type Provider = Core.Provider
export const CatalogModelStatus = Core.CatalogModelStatus
export type CatalogModelStatus = Core.CatalogModelStatus

export interface Interface extends Core.Interface {}

export class Service extends Context.Service<Service, Interface>()("@opencode/ModelsDev") {}

function baseURL(url: string | undefined, org: string | undefined) {
  if (!url) return
  const base = url.replace(/\/+$/, "")
  if (org) {
    if (base.includes("/api/organizations/")) return base
    if (base.endsWith("/api")) return `${base}/organizations/${org}`
    return `${base}/api/organizations/${org}`
  }
  if (base.includes("/openrouter")) return base
  if (base.endsWith("/api")) return `${base}/openrouter`
  return `${base}/api/openrouter`
}

export const layer: Layer.Layer<Service, never, Core.Service | Config.Service | Auth.Service | ModelCache.Service> =
  Layer.effect(
    Service,
    Effect.gen(function* () {
      const core = yield* Core.Service
      const config = yield* Config.Service
      const auth = yield* Auth.Service
      const cache = yield* ModelCache.Service

      const get = Effect.fn("ModelsDev.get")(function* () {
        const providers = overlay(yield* core.get())
        delete providers.cypher

        const cfg = yield* config.get()
        const disabled = new Set(cfg.disabled_providers ?? [])
        const enabled = cfg.enabled_providers ? new Set(cfg.enabled_providers) : undefined
        const allowed = (!enabled || enabled.has("cypher")) && !disabled.has("cypher")
        const apt = cfg.provider?.apertis?.options
        const aptURL = apt?.baseURL ?? "https://api.apertis.ai/v1"
        const aptOpts = apt?.baseURL ? { baseURL: apt.baseURL } : {}

        const addApertis = Effect.fnUntraced(function* () {
          if (providers.apertis) return
          const models = yield* cache.fetch("apertis", aptOpts).pipe(Effect.catch(() => Effect.succeed({})))
          providers.apertis = {
            id: "apertis",
            name: "Apertis",
            env: ["APERTIS_API_KEY"],
            api: aptURL,
            npm: "@ai-sdk/openai-compatible",
            models,
          }
          if (Object.keys(models).length === 0)
            yield* cache.refresh("apertis", aptOpts).pipe(Effect.ignore, Effect.forkDetach)
        })

        if (!allowed) {
          yield* addApertis()
          return providers
        }

        const opts = cfg.provider?.cypher?.options
        const info = yield* auth.get("cypher").pipe(Effect.catch(() => Effect.succeed(undefined)))
        const org = opts?.cypherOrganizationId ?? (info?.type === "oauth" ? info.accountId : undefined)
        const url = baseURL(opts?.baseURL, org)
        const fetch = {
          ...(url ? { baseURL: url } : {}),
          ...(org ? { cypherOrganizationId: org } : {}),
        }
        const models = yield* cache.fetch("cypher", fetch).pipe(Effect.catch(() => Effect.succeed({})))
        providers.cypher = {
          id: "cypher",
          name: "Cypher Gateway",
          env: ["CYPHER_API_KEY", "CYPHER_API_KEY"],
          api: CYPHER_OPENROUTER_BASE.endsWith("/") ? CYPHER_OPENROUTER_BASE : `${CYPHER_OPENROUTER_BASE}/`,
          npm: "@cypher/cypher-gateway",
          models,
        }
        if (Object.keys(models).length === 0) yield* cache.refresh("cypher", fetch).pipe(Effect.ignore, Effect.forkDetach)
        yield* addApertis()
        return providers
      })

      return Service.of({ get, refresh: core.refresh })
    }),
  )

export const defaultLayer = layer.pipe(
  Layer.provide(Core.defaultLayer),
  Layer.provide(Config.defaultLayer),
  Layer.provide(Auth.defaultLayer),
  Layer.provide(ModelCache.defaultLayer),
)

export { AI_SDK_PROVIDERS, PROMPTS }
export * as ModelsDev from "./models"
