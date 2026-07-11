import { Effect } from "effect"
import { HttpApiBuilder } from "effect/unstable/httpapi"
import { EffectBridge } from "@/effect/bridge"
import { InstanceHttpApi } from "@/server/routes/instance/httpapi/api"

export const indexingHandlers = HttpApiBuilder.group(InstanceHttpApi, "indexing", (handlers) =>
  Effect.gen(function* () {
    const mod = yield* Effect.promise(() => import("@/cypher/indexing"))
    const status = Effect.fn("IndexingHttpApi.status")(function* () {
      return yield* EffectBridge.fromPromise(() => mod.CypherIndexing.current())
    })
    const models = Effect.fn("IndexingHttpApi.models")(function* () {
      return yield* EffectBridge.fromPromise(() => mod.CypherIndexing.models())
    })
    const warnings = Effect.fn("IndexingHttpApi.warnings")(function* () {
      return yield* EffectBridge.fromPromise(() => mod.CypherIndexing.warnings())
    })

    return handlers.handle("status", status).handle("models", models).handle("warnings", warnings)
  }),
)
