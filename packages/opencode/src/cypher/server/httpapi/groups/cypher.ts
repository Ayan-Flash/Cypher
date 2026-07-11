import { Schema } from "effect"
import { HttpApi, HttpApiEndpoint, HttpApiError, HttpApiGroup, OpenApi } from "effect/unstable/httpapi"
import { Authorization } from "@/server/routes/instance/httpapi/middleware/authorization"
import { InstanceContextMiddleware } from "@/server/routes/instance/httpapi/middleware/instance-context"
import {
  WorkspaceRoutingMiddleware,
  WorkspaceRoutingQuery,
  WorkspaceRoutingQueryFields,
} from "@/server/routes/instance/httpapi/middleware/workspace-routing"
import { described } from "@/server/routes/instance/httpapi/groups/metadata"
import { AnacondaDesktopApi } from "./anaconda-desktop"
import { Result as AgentRequirementResult } from "@/cypher/agent-requirements"
import {
  Failure as NotebookFailure,
  Request as NotebookRequest,
  RequestID as NotebookRequestID,
  Result as NotebookResult,
} from "@/cypher/notebook/protocol"
import { ModelUsage } from "@/cypher/session/model-usage"
import { SessionID } from "@/session/schema"

const root = "/cypher"

export const RemoveSkillPayload = Schema.Struct({
  location: Schema.String,
})

export const RemoveAgentPayload = Schema.Struct({
  name: Schema.String,
})

export const AgentRequirementQuery = Schema.Struct({
  ...WorkspaceRoutingQueryFields,
  agent: Schema.String,
})
export const NotebookReplyPayload = Schema.Struct({ result: NotebookResult })
export const NotebookRejectPayload = Schema.Struct({ error: NotebookFailure })

export const CyphercodePaths = {
  heapSnapshot: `${root}/heap/snapshot`,
  agentRequirements: `${root}/agent/requirements`,
  removeSkill: `${root}/skill/remove`,
  removeAgent: `${root}/agent/remove`,
  notebookList: `${root}/notebook`,
  notebookReply: `${root}/notebook/:requestID/reply`,
  notebookReject: `${root}/notebook/:requestID/reject`,
  sessionModelUsage: `/session/:sessionID/model-usage`,
} as const

export const CyphercodeApi = HttpApi.make("cyphercode")
  .add(
    HttpApiGroup.make("cyphercode")
      .add(
        HttpApiEndpoint.post("heapSnapshot", CyphercodePaths.heapSnapshot, {
          query: WorkspaceRoutingQuery,
          success: described(Schema.String, "Heap snapshot file path"),
          error: HttpApiError.BadRequest,
        }).annotateMerge(
          OpenApi.annotations({
            identifier: "cypher.heap.snapshot",
            summary: "Write heap snapshot",
            description: "Write a heap snapshot for the CLI process to the log directory.",
          }),
        ),
        HttpApiEndpoint.get("agentRequirements", CyphercodePaths.agentRequirements, {
          query: AgentRequirementQuery,
          success: described(AgentRequirementResult, "Agent requirement status"),
        }).annotateMerge(
          OpenApi.annotations({
            identifier: "cypher.agentRequirements",
            summary: "Check agent requirements",
            description: "Check whether the selected agent's requirements are available in the request directory.",
          }),
        ),
        HttpApiEndpoint.post("removeSkill", CyphercodePaths.removeSkill, {
          query: WorkspaceRoutingQuery,
          payload: RemoveSkillPayload,
          success: described(Schema.Boolean, "Skill removed"),
          error: HttpApiError.BadRequest,
        }).annotateMerge(
          OpenApi.annotations({
            identifier: "cypher.removeSkill",
            summary: "Remove a skill",
            description: "Remove a skill by deleting its manifest from disk and clearing it from cache.",
          }),
        ),
        HttpApiEndpoint.post("removeAgent", CyphercodePaths.removeAgent, {
          query: WorkspaceRoutingQuery,
          payload: RemoveAgentPayload,
          success: described(Schema.Boolean, "Agent removed"),
          error: HttpApiError.BadRequest,
        }).annotateMerge(
          OpenApi.annotations({
            identifier: "cypher.removeAgent",
            summary: "Remove a custom agent",
            description:
              "Remove a custom (non-native) agent by deleting its markdown file from disk and refreshing state.",
          }),
        ),
        HttpApiEndpoint.get("notebookList", CyphercodePaths.notebookList, {
          query: WorkspaceRoutingQuery,
          success: described(Schema.Array(NotebookRequest), "Pending notebook host requests"),
        }).annotateMerge(
          OpenApi.annotations({
            identifier: "cypher.notebook.list",
            summary: "List pending notebook requests",
            description: "List pending native notebook requests for the routed workspace.",
          }),
        ),
        HttpApiEndpoint.post("notebookReply", CyphercodePaths.notebookReply, {
          params: { requestID: NotebookRequestID },
          query: WorkspaceRoutingQuery,
          payload: NotebookReplyPayload,
          success: described(Schema.Boolean, "Notebook reply accepted"),
          error: [HttpApiError.BadRequest, HttpApiError.NotFound],
        }).annotateMerge(
          OpenApi.annotations({
            identifier: "cypher.notebook.reply",
            summary: "Reply to a notebook request",
            description: "Complete a pending native notebook request with a structured result.",
          }),
        ),
        HttpApiEndpoint.post("notebookReject", CyphercodePaths.notebookReject, {
          params: { requestID: NotebookRequestID },
          query: WorkspaceRoutingQuery,
          payload: NotebookRejectPayload,
          success: described(Schema.Boolean, "Notebook rejection accepted"),
          error: HttpApiError.NotFound,
        }).annotateMerge(
          OpenApi.annotations({
            identifier: "cypher.notebook.reject",
            summary: "Reject a notebook request",
            description: "Complete a pending native notebook request with a structured host error.",
          }),
        ),
        HttpApiEndpoint.get("sessionModelUsage", CyphercodePaths.sessionModelUsage, {
          params: { sessionID: SessionID },
          query: WorkspaceRoutingQuery,
          success: described(ModelUsage.Info, "Model usage for a session tree"),
          error: HttpApiError.NotFound,
        }).annotateMerge(
          OpenApi.annotations({
            identifier: "cypher.sessionModelUsage",
            summary: "Get session model usage",
            description: "Get token usage and direct cost by model for the complete top-level session tree.",
          }),
        ),
      )
      .annotateMerge(
        OpenApi.annotations({
          title: "cypher",
          description: "Cypher-specific routes.",
        }),
      )
      .middleware(InstanceContextMiddleware)
      .middleware(WorkspaceRoutingMiddleware)
      .middleware(Authorization),
  )
  .addHttpApi(AnacondaDesktopApi)
  .annotateMerge(
    OpenApi.annotations({
      title: "cypher HttpApi",
      version: "0.0.1",
      description: "Cypher HttpApi surface.",
    }),
  )
