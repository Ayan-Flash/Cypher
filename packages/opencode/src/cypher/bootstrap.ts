import { Cause, Context, Effect, Layer } from "effect"
import { EffectBridge } from "@/effect/bridge"
import { CypherSessions } from "@/cypher-sessions/cypher-sessions"
import * as Log from "@opencode-ai/core/util/log"
import { Global } from "@opencode-ai/core/global"
import { InstallationVersion } from "@opencode-ai/core/installation/version"
import path from "node:path"
import { Bus } from "@/bus"
import { Provider } from "@/provider/provider"
import { Session } from "@/session/session"
import { SessionSummary } from "@/session/summary"
import { SessionExport } from "@/cypher/session-export"
import { createWorkspaceProvider } from "@/cypher/session-export/workspace-provider"
import { Instance } from "@/cypher/instance"
import { Identity } from "@cypher/cypher-telemetry"
import { MemoryLifecycle } from "@/cypher/memory/turn"
import { MemoryService } from "@cypher/cypher-memory/effect/service"
import { MemoryEvents } from "@/cypher/memory/events"
import { installMemoryRuntime } from "@/cypher/memory/runtime"
import { CypherToolRegistry } from "@/cypher/tool/registry"

const log = Log.create({ service: "cypher-bootstrap" })

export namespace CyphercodeBootstrap {
  export interface Interface {
    readonly init: () => Effect.Effect<void, unknown>
  }

  export class Service extends Context.Service<Service, Interface>()("@cypher/Bootstrap") {}

  export const layer = Layer.effect(
    Service,
    Effect.gen(function* () {
      // Bind the package memory effect layer to opencode (paths, instance binder, logger, event sink).
      installMemoryRuntime()
      const cypher = yield* CypherSessions.Service
      const bus = yield* Bus.Service
      const sessions = yield* Session.Service
      const summary = yield* SessionSummary.Service
      const provider = yield* Provider.Service
      const memory = yield* MemoryService.Service

      const init = Effect.fn("CyphercodeBootstrap.init")(function* () {
        yield* cypher.init()
        yield* MemoryLifecycle.subscribe({ bus, sessions, summary, provider, memory })
        // Invalidate enabled cache on every memory state mutation (properties.directory holds the memory root).
        yield* bus.subscribeCallback(MemoryEvents.Status, (evt) =>
          CypherToolRegistry.invalidateMemoryEnabled(evt.properties.directory),
        )
        yield* bus.subscribeCallback(MemoryEvents.Updated, (evt) =>
          CypherToolRegistry.invalidateMemoryEnabled(evt.properties.directory),
        )
        // cypher_change start - session export bootstrap
        yield* Effect.gen(function* () {
          const anon = yield* EffectBridge.fromPromise(() =>
            Identity.getMachineId().catch((err) => {
              log.warn("session export identity failed", { err })
              return undefined
            }),
          )
          SessionExport.init({
            agentVersion: InstallationVersion,
            anonId: anon,
            dbPath: path.join(Global.Path.data, "session-export.db"),
            workspaceKey: Instance.directory,
            subscribeAll: (cb) => Bus.subscribeAll(cb),
            snapshotProvider: createWorkspaceProvider({
              root: Instance.directory,
              statePath: path.join(Global.Path.data, "session-export-workspace.json"),
            }),
          })
        }).pipe(
          Effect.catchCause((cause) =>
            Effect.sync(() => log.warn("session export bootstrap failed", { err: Cause.squash(cause) })),
          ),
        )
        // cypher_change end
        yield* EffectBridge.fromPromise(() =>
          import("@/cypher/indexing").then((mod) => mod.CypherIndexing.init()),
        ).pipe(
          Effect.catchCause((cause) =>
            Effect.sync(() => log.warn("indexing bootstrap failed", { err: Cause.squash(cause) })),
          ),
          Effect.forkDetach,
        )
      })

      return Service.of({ init })
    }),
  )

  export const defaultLayer = layer.pipe(
    Layer.provide([
      CypherSessions.defaultLayer,
      Session.defaultLayer,
      SessionSummary.defaultLayer,
      Provider.defaultLayer,
      MemoryService.layer,
      Bus.defaultLayer,
    ]),
  )
}
