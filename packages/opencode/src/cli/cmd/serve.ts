import { Effect } from "effect"
import { Server } from "../../server/server"
import { effectCmd } from "../effect-cmd"
import { withNetworkOptions, resolveNetworkOptions } from "../network"
import { Flag } from "@opencode-ai/core/flag/flag"
import { InstanceRuntime } from "../../project/instance-runtime" // cypher_change
import { startParentWatchdog } from "../../cypher/parent-watchdog" // cypher_change

export const ServeCommand = effectCmd({
  command: "serve",
  builder: (yargs) => withNetworkOptions(yargs),
  describe: "starts a headless cypher server",
  // Server loads instances per-request via x-cypher-directory header — no
  // need for an ambient project InstanceContext at startup.
  instance: false, // cypher_change
  handler: Effect.fn("Cli.serve")(function* (args) {
    if (!Flag.CYPHER_SERVER_PASSWORD) {
      console.log("Warning: CYPHER_SERVER_PASSWORD is not set; server is unsecured.")
    }
    const opts = yield* resolveNetworkOptions(args)
    const server = yield* Effect.promise(() => Server.listen(opts))

    // cypher_change start
    const urls = server.urls

    console.log(`cypher server listening on ${urls.bind}`)
    if (urls.local !== urls.bind) console.log(`  Local:   ${urls.local}`)
    if (urls.network) console.log(`  Network: ${urls.network}`)
    // cypher_change end

    // cypher_change start - graceful signal shutdown
    // yield* Effect.never
    yield* Effect.promise(
      () =>
        new Promise<void>((resolve) => {
          // Exit if the editor client that spawned us is hard-killed (no signal reaches us).
          const stopWatchdog = startParentWatchdog(() => process.kill(process.pid, "SIGTERM"))
          const shutdown = async () => {
            stopWatchdog()
            try {
              await InstanceRuntime.disposeAllInstances()
              await server.stop(true)
            } finally {
              resolve()
            }
          }
          process.once("SIGTERM", shutdown)
          process.once("SIGINT", shutdown)
          process.once("SIGHUP", shutdown)
        }),
    )
    // cypher_change end
  }),
})
