// cypher_change - new file
// When the injected Cypher models source returns a 401 error result, ModelCache surfaces
// the failure and caches empty models (allowing re-auth via /connect).
// The real fetchCypherModels 401-fallback unit test lives in packages/cypher-gateway/test/api/models.test.ts.

import { expect } from "bun:test"
import { Effect, Layer } from "effect"
import { FetchHttpClient } from "effect/unstable/http"
import * as Log from "@opencode-ai/core/util/log"

Log.init({ print: false })

import { Auth } from "../../src/auth"
import { ModelCache } from "../../src/provider/model-cache"
import { TestConfig } from "../fixture/config"
import { testEffect } from "../lib/effect"

const auth = Layer.mock(Auth.Service)({
  get: () => Effect.succeed(undefined),
})

const models = Layer.succeed(
  ModelCache.CypherModelsService,
  ModelCache.CypherModelsService.of({
    fetch: () => Effect.succeed({ models: {}, error: { kind: "unauthorized", status: 401 } }),
  }),
)

const layer = Layer.fresh(ModelCache.layer).pipe(
  Layer.provide(FetchHttpClient.layer),
  Layer.provide(TestConfig.layer()),
  Layer.provide(auth),
  Layer.provide(models),
)

const it = testEffect(layer)

it.live("401 from Cypher models sets provider as failed in ModelCache", () =>
  Effect.gen(function* () {
    const cache = yield* ModelCache.Service
    yield* cache.fetch("cypher")
    expect(yield* cache.failedProviders()).toContain("cypher")
    expect(yield* cache.getFailure("cypher")).toMatchObject({ kind: "unauthorized", status: 401 })
  }),
)

it.live("401 from Cypher models caches empty models (not undefined)", () =>
  Effect.gen(function* () {
    const cache = yield* ModelCache.Service
    yield* cache.fetch("cypher")
    expect(yield* cache.get("cypher")).toEqual({})
  }),
)
