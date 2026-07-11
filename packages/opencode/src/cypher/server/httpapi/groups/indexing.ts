import { Schema } from "effect"
import { HttpApi, HttpApiEndpoint, HttpApiGroup, OpenApi } from "effect/unstable/httpapi"
import { IndexingStatusInfo, IndexingWarningInfo } from "@/cypher/indexing-event"
import { Authorization } from "@/server/routes/instance/httpapi/middleware/authorization"
import { InstanceContextMiddleware } from "@/server/routes/instance/httpapi/middleware/instance-context"
import {
  WorkspaceRoutingMiddleware,
  WorkspaceRoutingQuery,
} from "@/server/routes/instance/httpapi/middleware/workspace-routing"
import { described } from "@/server/routes/instance/httpapi/groups/metadata"

export { IndexingStatusInfo, IndexingStatusState, IndexingWarningInfo } from "@/cypher/indexing-event"

export const CypherEmbeddingModel = Schema.Struct({
  id: Schema.String,
  name: Schema.String,
  dimension: Schema.Int.check(Schema.isGreaterThan(0)),
  scoreThreshold: Schema.Finite.check(Schema.isBetween({ minimum: 0, maximum: 1 })),
  note: Schema.optional(Schema.String),
})

export const CypherEmbeddingModelCatalog = Schema.Struct({
  defaultModel: Schema.String,
  models: Schema.Array(CypherEmbeddingModel),
  aliases: Schema.Record(Schema.String, Schema.String),
}).annotate({ identifier: "CypherEmbeddingModelCatalog" })

const root = "/indexing"

export const IndexingPaths = {
  status: `${root}/status`,
  models: `${root}/models`,
  warnings: `${root}/warnings`,
} as const

export const IndexingApi = HttpApi.make("indexing")
  .add(
    HttpApiGroup.make("indexing")
      .add(
        HttpApiEndpoint.get("status", IndexingPaths.status, {
          query: WorkspaceRoutingQuery,
          success: described(IndexingStatusInfo, "Indexing status"),
        }).annotateMerge(
          OpenApi.annotations({
            identifier: "indexing.status",
            summary: "Get indexing status",
            description: "Retrieve the current code indexing status for the active project.",
          }),
        ),
        HttpApiEndpoint.get("warnings", IndexingPaths.warnings, {
          query: WorkspaceRoutingQuery,
          success: described(Schema.Array(IndexingWarningInfo), "Indexing warnings"),
        }).annotateMerge(
          OpenApi.annotations({
            identifier: "indexing.warnings",
            summary: "Get indexing warnings",
            description: "Retrieve code indexing warnings for the active project.",
          }),
        ),
      )
      .add(
        HttpApiEndpoint.get("models", IndexingPaths.models, {
          query: WorkspaceRoutingQuery,
          success: described(CypherEmbeddingModelCatalog, "Cypher embedding model catalog"),
        }).annotateMerge(
          OpenApi.annotations({
            identifier: "indexing.models",
            summary: "List Cypher embedding models",
            description: "Retrieve the embedding models available through the active Cypher account.",
          }),
        ),
      )
      .annotateMerge(
        OpenApi.annotations({
          title: "indexing",
          description: "Cypher indexing routes.",
        }),
      )
      .middleware(InstanceContextMiddleware)
      .middleware(WorkspaceRoutingMiddleware)
      .middleware(Authorization),
  )
  .annotateMerge(
    OpenApi.annotations({
      title: "cypher HttpApi",
      version: "0.0.1",
      description: "Cypher HttpApi surface.",
    }),
  )
