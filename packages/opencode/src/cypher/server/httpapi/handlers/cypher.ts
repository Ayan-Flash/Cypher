import { Effect } from "effect"
import { HttpApiBuilder, HttpApiError } from "effect/unstable/httpapi"
import * as CypherAgent from "@/cypher/agent"
import * as CypherSkill from "@/cypher/skill-remove"
import { Agent } from "@/agent/agent"
import { Config } from "@/config/config"
import { InstanceState } from "@/effect/instance-state"
import { HeapSnapshot } from "@/cypher/cli/heap-snapshot"
import type { RequestID as NotebookRequestID } from "@/cypher/notebook/protocol"
import { Notebook } from "@/cypher/notebook/service"
import { ModelUsage } from "@/cypher/session/model-usage"
import { InstanceStore } from "@/project/instance-store"
import { InstanceHttpApi } from "@/server/routes/instance/httpapi/api"
import { Skill } from "@/skill"
import type { SessionID } from "@/session/schema"
import { NotebookRejectPayload, NotebookReplyPayload, RemoveAgentPayload, RemoveSkillPayload } from "../groups/cypher"

export const cypherHandlers = HttpApiBuilder.group(InstanceHttpApi, "cyphercode", (handlers) =>
  Effect.gen(function* () {
    const agents = yield* Agent.Service
    const skills = yield* Skill.Service
    const config = yield* Config.Service
    const store = yield* InstanceStore.Service
    const notebook = yield* Notebook.Service

    const heapSnapshot = Effect.fn("CyphercodeHttpApi.heapSnapshot")(function* () {
      return yield* Effect.sync(() => HeapSnapshot.write())
    })

    const agentRequirements = Effect.fn("CyphercodeHttpApi.agentRequirements")(function* (ctx: {
      query: { agent: string }
    }) {
      return yield* agents.requirementStatus(ctx.query.agent)
    })

    const removeSkill = Effect.fn("CyphercodeHttpApi.removeSkill")(function* (ctx: {
      payload: typeof RemoveSkillPayload.Type
    }) {
      const instance = yield* InstanceState.context
      const entries = yield* skills.all()
      yield* Effect.tryPromise({
        try: () => CypherSkill.remove(ctx.payload.location, entries),
        catch: () => new HttpApiError.BadRequest({}),
      })
      yield* store.dispose(instance)
      return true
    })

    const removeAgent = Effect.fn("CyphercodeHttpApi.removeAgent")(function* (ctx: {
      payload: typeof RemoveAgentPayload.Type
    }) {
      const instance = yield* InstanceState.context
      const agent = yield* agents.get(ctx.payload.name)
      const dirs = yield* config.directories()
      yield* Effect.tryPromise({
        try: () => CypherAgent.remove({ name: ctx.payload.name, agent, dirs, directory: instance.directory }),
        catch: (err) => err,
      }).pipe(
        Effect.catch((err) => {
          if (CypherAgent.RemoveError.isInstance(err)) return Effect.fail(new HttpApiError.BadRequest({}))
          return Effect.die(err)
        }),
      )
      yield* store.dispose(instance)
      return true
    })

    const notebookList = Effect.fn("CyphercodeHttpApi.notebookList")(function* () {
      return yield* notebook.list()
    })

    const notebookReply = Effect.fn("CyphercodeHttpApi.notebookReply")(function* (ctx: {
      params: { requestID: NotebookRequestID }
      payload: typeof NotebookReplyPayload.Type
    }) {
      yield* notebook.reply({ requestID: ctx.params.requestID, result: ctx.payload.result }).pipe(
        Effect.catchTag("Notebook.NotFoundError", () => Effect.fail(new HttpApiError.NotFound({}))),
        Effect.catchTag("Notebook.InvalidReplyError", () => Effect.fail(new HttpApiError.BadRequest({}))),
      )
      return true
    })

    const notebookReject = Effect.fn("CyphercodeHttpApi.notebookReject")(function* (ctx: {
      params: { requestID: NotebookRequestID }
      payload: typeof NotebookRejectPayload.Type
    }) {
      yield* notebook
        .reject({ requestID: ctx.params.requestID, error: ctx.payload.error })
        .pipe(Effect.catchTag("Notebook.NotFoundError", () => Effect.fail(new HttpApiError.NotFound({}))))
      return true
    })

    const sessionModelUsage = Effect.fn("CyphercodeHttpApi.sessionModelUsage")(function* (ctx: {
      params: { sessionID: SessionID }
    }) {
      const usage = yield* ModelUsage.get(ctx.params.sessionID)
      if (!usage) return yield* new HttpApiError.NotFound({})
      return usage
    })

    return handlers
      .handle("heapSnapshot", heapSnapshot)
      .handle("agentRequirements", agentRequirements)
      .handle("removeSkill", removeSkill)
      .handle("removeAgent", removeAgent)
      .handle("notebookList", notebookList)
      .handle("notebookReply", notebookReply)
      .handle("notebookReject", notebookReject)
      .handle("sessionModelUsage", sessionModelUsage)
  }),
)
