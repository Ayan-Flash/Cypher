import { Config } from "@/config/config"
// cypher_change start - preserve Cypher API default model overlay
import { fetchDefaultModel } from "@cypher/cypher-gateway"
import { Auth } from "@/auth"
import { ModelID, ProviderID } from "@/provider/schema"
import { filterPromptTrainingModels, nonEmptyProviders } from "@/cypher/provider/model-filter"
// cypher_change end
import { Provider } from "@/provider/provider"
import * as InstanceState from "@/effect/instance-state"
import { Effect } from "effect"
import { HttpApiBuilder, HttpApiError } from "effect/unstable/httpapi" // cypher_change
import { InstanceHttpApi } from "../api"
import { markInstanceForDisposal } from "../lifecycle"

export const configHandlers = HttpApiBuilder.group(InstanceHttpApi, "config", (handlers) =>
  Effect.gen(function* () {
    const providerSvc = yield* Provider.Service
    const configSvc = yield* Config.Service

    const get = Effect.fn("ConfigHttpApi.get")(function* () {
      return yield* configSvc.get()
    })

    const update = Effect.fn("ConfigHttpApi.update")(function* (ctx) {
      yield* configSvc.update(ctx.payload)
      yield* markInstanceForDisposal(yield* InstanceState.context)
      return ctx.payload
    })

    // cypher_change start
    const warnings = Effect.fn("ConfigHttpApi.warnings")(function* () {
      return yield* configSvc.warnings()
    })
    // cypher_change end

    const providers = Effect.fn("ConfigHttpApi.providers")(function* () {
      // cypher_change start
      const config = yield* configSvc.get()
      const providers = filterPromptTrainingModels(
        yield* providerSvc.list(),
        config.hide_prompt_training_models === true,
      )
      const defaults = Provider.defaultModelIDs(nonEmptyProviders(providers))
      // cypher_change end

      // cypher_change start - Fetch default model from Cypher API when the cypher provider is available.
      if (providers[ProviderID.cypher]) {
        const auth = yield* Auth.Service
        const info = yield* auth.get("cypher").pipe(Effect.mapError(() => new HttpApiError.Unauthorized({}))) // cypher_change
        const token = info?.type === "oauth" ? info.access : info?.key
        const organizationId = info?.type === "oauth" ? info.accountId : undefined
        const model = yield* Effect.promise(() => fetchDefaultModel(token, organizationId))
        if (model && providers[ProviderID.cypher]?.models[model]) defaults[ProviderID.cypher] = ModelID.make(model)
      }
      // cypher_change end

      return {
        providers: Object.values(providers).map(Provider.toPublicInfo),
        default: defaults,
      }
    })

    return handlers
      .handle("get", get)
      .handle("update", update)
      .handle("warnings", warnings)
      .handle("providers", providers) // cypher_change
  }),
)
