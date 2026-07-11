import * as Tool from "./tool"
import DESCRIPTION from "./task.txt"
import { ToolJsonSchema } from "./json-schema"
import { BackgroundJob } from "@/background/job"
import { Session } from "@/session/session"
import { SessionID, MessageID } from "../session/schema"
import { MessageV2 } from "../session/message-v2"
import { Agent } from "../agent/agent"
import { deriveSubagentSessionPermission } from "../agent/subagent-permissions"
import type { SessionPrompt } from "../session/prompt"
import { Config } from "@/config/config"
import { Provider } from "@/provider/provider" // cypher_change
import { CypherTask } from "../cypher/tool/task" // cypher_change
import { CypherTaskBackgroundProcess } from "../cypher/tool/task-background-process" // cypher_change
import { CypherCostPropagation } from "../cypher/session/cost-propagation" // cypher_change
import { CypherSessionProcessor } from "../cypher/session/processor" // cypher_change
import { CypherSession } from "../cypher/session" // cypher_change
import { errorMessage } from "@/util/error" // cypher_change
import { Cause, Effect, Exit, Schema, Scope } from "effect"
import { EffectBridge } from "@/effect/bridge"
import { RuntimeFlags } from "@/effect/runtime-flags"
import * as SandboxPolicy from "@/cypher/sandbox/policy" // cypher_change

export interface TaskPromptOps {
  cancel(sessionID: SessionID): Effect.Effect<void>
  resolvePromptParts(template: string): Effect.Effect<SessionPrompt.PromptInput["parts"]>
  prompt(input: SessionPrompt.PromptInput): Effect.Effect<MessageV2.WithParts>
}

const id = "task"
const BACKGROUND_DESCRIPTION = [
  "",
  "",
  [
    "Background mode: background=true launches the subagent asynchronously and returns immediately.",
    "Foreground is the default; use it when you need the result before continuing.",
    "Use background only for independent work that can run while you continue elsewhere.",
    "You will be notified automatically when it finishes.",
  ].join(" "),
].join("\n")

const BaseParameterFields = {
  description: Schema.String.annotate({ description: "A short (3-5 words) description of the task" }),
  prompt: Schema.String.annotate({ description: "The task for the agent to perform" }),
  subagent_type: Schema.String.annotate({ description: "The type of specialized agent to use for this task" }),
  task_id: Schema.optional(Schema.String).annotate({
    description:
      "This should only be set if you mean to resume a previous task (you can pass a prior task_id and the task will continue the same subagent session as before instead of creating a fresh one)",
  }),
  command: Schema.optional(Schema.String).annotate({ description: "The command that triggered this task" }),
}

const BaseParameters = Schema.Struct(BaseParameterFields)

export const Parameters = Schema.Struct({
  ...BaseParameterFields,
  background: Schema.optional(Schema.Boolean).annotate({
    description: "Run the agent in the background. You will be notified when it completes.",
  }),
})

function output(sessionID: SessionID, text: string) {
  return [`<task id="${sessionID}" state="completed">`, "<task_result>", text, "</task_result>", "</task>"].join("\n")
}

function backgroundOutput(sessionID: SessionID) {
  return [
    `<task id="${sessionID}" state="running">`,
    "<summary>Background task started</summary>",
    "<task_result>",
    "Background task started. You will be notified automatically when it finishes; do not poll for progress.",
    "Do not duplicate its work. Continue only with non-overlapping work, or stop if there is nothing else useful to do.",
    "</task_result>",
    "</task>",
  ].join("\n")
}

function backgroundMessage(input: {
  sessionID: SessionID
  description: string
  state: "completed" | "error"
  text: string
}) {
  const tag = input.state === "completed" ? "task_result" : "task_error"
  const title =
    input.state === "completed"
      ? `Background task completed: ${input.description}`
      : `Background task failed: ${input.description}`
  // cypher_change start - surface the resumable task_id when a background subagent fails (#11620)
  const hint = resumeHint(input.sessionID)
  const body =
    input.state === "error" && !input.text.includes(hint)
      ? `${input.text}\n${hint}`
      : input.text
  // cypher_change end
  return [
    `<task id="${input.sessionID}" state="${input.state}">`,
    `<summary>${title}</summary>`,
    `<${tag}>`,
    body, // cypher_change - was input.text
    `</${tag}>`,
    "</task>",
  ].join("\n")
}

function errorText(error: unknown) {
  if (error instanceof Error) return error.message
  return String(error)
}

// cypher_change start - tell the parent agent how to resume a stopped/failed subagent (#11620)
function resumeHint(sessionID: SessionID) {
  return [
    `This subagent session can be resumed: call the task tool again with task_id="${sessionID}"`,
    `and a prompt describing how to continue or recover. Its prior context is preserved.`,
  ].join(" ")
}
// cypher_change end

export const TaskTool = Tool.define(
  id,
  Effect.gen(function* () {
    const agent = yield* Agent.Service
    const background = yield* BackgroundJob.Service
    const config = yield* Config.Service
    const sessions = yield* Session.Service
    const provider = yield* Provider.Service // cypher_change
    const scope = yield* Scope.Scope
    const flags = yield* RuntimeFlags.Service

    const run = Effect.fn("TaskTool.execute")(function* (
      params: Schema.Schema.Type<typeof Parameters>,
      ctx: Tool.Context,
    ) {
      const cfg = yield* config.get()
      const runInBackground = params.background === true
      if (runInBackground && !flags.experimentalBackgroundSubagents) {
        return yield* Effect.fail(new Error("Background subagents require CYPHER_EXPERIMENTAL_BACKGROUND_SUBAGENTS=true"))
      }

      if (!ctx.extra?.bypassAgentCheck) {
        yield* ctx.ask({
          permission: id,
          patterns: [params.subagent_type],
          always: ["*"],
          metadata: {
            description: params.description,
            subagent_type: params.subagent_type,
          },
        })
      }

      const next = yield* agent.get(params.subagent_type)
      if (!next) {
        return yield* Effect.fail(new Error(`Unknown agent type: ${params.subagent_type} is not a valid agent type`))
      }
      // cypher_change start — reject primary agents; only subagent/all modes allowed
      CypherTask.validate(next, params.subagent_type)
      // cypher_change end

      const canTask = CypherTask.nestedTask() // cypher_change - Cypher disallows subagents spawning subagents
      const canTodo = next.permission.some((rule) => rule.permission === "todowrite")

      const session = params.task_id
        ? yield* sessions.get(SessionID.make(params.task_id)).pipe(Effect.catchCause(() => Effect.succeed(undefined)))
        : undefined
      if (session && session.parentID !== ctx.sessionID) {
        return yield* Effect.fail(
          new Error(`Cannot resume session ${params.task_id}: not a child of the current session`),
        ) // cypher_change - prevent cross-session task resume
      }
      const parent = yield* sessions.get(ctx.sessionID)
      const parentAgent = parent.agent
        ? yield* agent.get(parent.agent).pipe(Effect.catchCause(() => Effect.succeed(undefined)))
        : undefined
      // cypher_change start — inherit edit/bash/MCP restrictions from calling agent
      const caller = yield* agent.get(ctx.agent)
      const rules = CypherTask.inherited({ caller, session: parent, mcp: cfg.mcp })
      // cypher_change end
      // cypher_change start - refresh current parent restrictions when resuming an existing task session
      const fallback = SandboxPolicy.fallback(cfg)
      if (session) {
        yield* SandboxPolicy.inherit(ctx.sessionID, session.id, fallback)
        const permission = CypherTask.merge(
          session.permission ?? [],
          deriveSubagentSessionPermission({
            parentSessionPermission: parent.permission ?? [],
            parentAgent,
            subagent: next,
          }),
          CypherTask.permissions(rules),
        )
        session.permission = permission
        yield* sessions.setPermission({ sessionID: session.id, permission })
      }
      // cypher_change end
      const platform = CypherSession.resolvePlatform(ctx.sessionID) // cypher_change - preserve parent attribution across task creation/resume
      // cypher_change start - create a child session with inherited Cypher restrictions
      const nextSession =
        session ??
        (yield* sessions.create({
          parentID: ctx.sessionID,
          title: params.description + ` (@${next.name} subagent)`,
          platform, // cypher_change
          // cypher_change start - dedupe inherited restrictions before child prompt toggles persist
          permission: CypherTask.merge(
            deriveSubagentSessionPermission({
              parentSessionPermission: parent.permission ?? [],
              parentAgent,
              subagent: next,
            }),
            cfg.experimental?.primary_tools?.map((item) => ({
              pattern: "*",
              action: "allow" as const,
              permission: item,
            })) ?? [],
            CypherTask.permissions(rules),
          ),
          // cypher_change end
        }))
      // cypher_change end
      // cypher_change start - rebuild in-memory ancestry and inherit confinement after creation/resume
      CypherSession.register({ id: nextSession.id, parentID: ctx.sessionID, platform })
      yield* SandboxPolicy.inherit(ctx.sessionID, nextSession.id, fallback)
      // cypher_change end

      const msg = yield* MessageV2.get({ sessionID: ctx.sessionID, messageID: ctx.messageID }).pipe(Effect.orDie)
      if (msg.info.role !== "assistant") return yield* Effect.fail(new Error("Not an assistant message"))

      // cypher_change start — prefer valid subagent overrides, safely inheriting when overrides go stale
      const selected = yield* CypherTask.resolveModel({
        name: next.name,
        agent: next,
        config: cfg,
        parent: {
          modelID: msg.info.modelID,
          providerID: msg.info.providerID,
        },
        variant: msg.info.variant,
        provider,
      })
      const model = selected.model
      const variant = selected.variant
      // cypher_change end
      const metadata = {
        parentSessionId: ctx.sessionID,
        sessionId: nextSession.id,
        model,
        variant, // cypher_change
        ...(runInBackground ? { background: true } : {}),
      }

      yield* ctx.metadata({
        title: params.description,
        metadata,
      })

      const ops = ctx.extra?.promptOps as TaskPromptOps
      if (!ops) return yield* Effect.fail(new Error("TaskTool requires promptOps in ctx.extra"))

      const runTask = Effect.fn("TaskTool.runTask")(function* () {
        const parts = yield* ops.resolvePromptParts(params.prompt)
        CypherSessionProcessor.markReviewTelemetry(parts, params.command) // cypher_change - carry review command into child session telemetry
        const result = yield* ops.prompt({
          messageID: MessageID.ascending(),
          sessionID: nextSession.id,
          model: {
            modelID: model.modelID,
            providerID: model.providerID,
          },
          variant, // cypher_change
          agent: next.name,
          tools: {
            question: false, // cypher_change - subagents cannot prompt the user directly
            interactive_terminal: false, // cypher_change - subagents cannot take over the user's terminal
            ...(canTodo ? {} : { todowrite: false }),
            ...(canTask ? {} : { task: false }),
            ...Object.fromEntries((cfg.experimental?.primary_tools ?? []).map((item) => [item, false])),
          },
          parts,
        })
        // cypher_change start - expose terminal child assistant errors through the task tool boundary,
        // including the resumable task_id so the parent agent can continue the subagent (#11620)
        if (result.info.role === "assistant" && result.info.error) {
          return yield* Effect.fail(
            new Error(`${errorMessage(result.info.error)}\n${resumeHint(nextSession.id)}`),
          )
        }
        // cypher_change end
        return result.parts.findLast((item) => item.type === "text")?.text ?? ""
      }, Effect.ensuring(CypherTaskBackgroundProcess.finish(nextSession.id))) // cypher_change - transfer inherited processes when the child run ends

      // cypher_change start - inject completed background task results into the parent session
      const inject = Effect.fn("TaskTool.injectBackgroundResult")(function* (
        state: "completed" | "error",
        text: string,
      ) {
        const currentParent = yield* sessions.get(ctx.sessionID)
        yield* ops
          .prompt({
            sessionID: ctx.sessionID,
            agent: currentParent.agent ?? ctx.agent,
            parts: [
              {
                type: "text",
                synthetic: true,
                text: backgroundMessage({
                  sessionID: nextSession.id,
                  description: params.description,
                  state,
                  text,
                }),
              },
            ],
          })
          .pipe(Effect.ignore, Effect.forkIn(scope, { startImmediately: true }))
      })
      // cypher_change end

      const existing = yield* background.get(nextSession.id)
      if (existing?.status === "running") {
        return yield* Effect.fail(new Error(`Task ${nextSession.id} is already running.`))
      }

      if (runInBackground) {
        const info = yield* background.start({
          id: nextSession.id,
          type: id,
          title: params.description,
          metadata,
          // cypher_change start - background tasks propagate only cost accrued by this invocation
          run: Effect.acquireUseRelease(
            CypherCostPropagation.childCost(sessions, nextSession.id),
            () =>
              runTask().pipe(
                Effect.tap((text) => inject("completed", text).pipe(Effect.ignore)),
                Effect.catchCause((cause) =>
                  (Cause.hasInterruptsOnly(cause)
                    ? Effect.void
                    : inject("error", errorText(Cause.squash(cause))).pipe(Effect.ignore)
                  ).pipe(Effect.andThen(Effect.failCause(cause))),
                ),
              ),
            (costBefore) =>
              Effect.gen(function* () {
                const costAfter = yield* CypherCostPropagation.childCost(sessions, nextSession.id)
                yield* CypherCostPropagation.propagate(sessions, ctx.sessionID, ctx.messageID, costAfter - costBefore)
              }),
          ),
          // cypher_change end
        })

        return {
          title: params.description,
          metadata: {
            ...metadata,
            jobId: info.id,
          },
          output: backgroundOutput(nextSession.id),
        }
      }

      const runCancel = yield* EffectBridge.make()
      const cancel = ops.cancel(nextSession.id)

      function onAbort() {
        runCancel.fork(cancel)
      }

      return yield* Effect.acquireUseRelease(
        // cypher_change start - snapshot child cost so we propagate only the delta on resume (#6321)
        Effect.gen(function* () {
          ctx.abort.addEventListener("abort", onAbort)
          return yield* CypherCostPropagation.childCost(sessions, nextSession.id)
        }),
        // cypher_change end
        () =>
          Effect.gen(function* () {
            const text = yield* runTask()
            return {
              title: params.description,
              metadata,
              output: output(nextSession.id, text),
            }
          }),
        // cypher_change start - propagate subagent cost delta to parent on every exit path (#6321)
        (costBefore, exit) =>
          Effect.gen(function* () {
            if (Exit.hasInterrupts(exit)) yield* cancel
          }).pipe(
            Effect.ensuring(
              Effect.gen(function* () {
                ctx.abort.removeEventListener("abort", onAbort)
                const costAfter = yield* CypherCostPropagation.childCost(sessions, nextSession.id).pipe(
                  Effect.catchTag("NotFoundError", () => Effect.succeed(costBefore)),
                )
                yield* CypherCostPropagation.propagate(
                  sessions,
                  ctx.sessionID,
                  ctx.messageID,
                  costAfter - costBefore,
                ).pipe(Effect.catchTag("NotFoundError", () => Effect.void))
              }),
            ),
          ),
        // cypher_change end
      )
    })

    return {
      description: flags.experimentalBackgroundSubagents ? DESCRIPTION + BACKGROUND_DESCRIPTION : DESCRIPTION,
      parameters: Parameters,
      jsonSchema: flags.experimentalBackgroundSubagents ? undefined : ToolJsonSchema.fromSchema(BaseParameters),
      execute: (params: Schema.Schema.Type<typeof Parameters>, ctx: Tool.Context) =>
        run(params, ctx).pipe(Effect.orDie),
    }
  }),
)
