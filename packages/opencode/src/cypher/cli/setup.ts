import type { Argv } from "yargs"
import * as Log from "@opencode-ai/core/util/log"
import { Global } from "@opencode-ai/core/global"
import { InstallationBuildKind, InstallationVersion } from "@opencode-ai/core/installation/version"
import { Telemetry } from "@cypher/cypher-telemetry"
import { migrateLegacyCypherAuth, ENV_FEATURE, ENV_VERSION } from "@cypher/cypher-gateway"
import { AppRuntime } from "@/effect/app-runtime"
import { Config } from "@/config/config"
import { Auth } from "@/auth"
import { InstanceRuntime } from "@/project/instance-runtime"
import { SessionExport } from "@/cypher/session-export"
import { CypherShutdown } from "@/cypher/cli/shutdown"
import { createHelpCommand } from "@/cypher/help-command"
import { CypherConsoleCommand } from "@/cypher/cli/cmd/console"
import { RollCallCommand } from "@/cypher/cli/cmd/roll-call"
import { ProfileCommand } from "@/cypher/cli/cmd/profile"
import { DaemonCommand } from "@/cypher/cli/cmd/daemon"
import { DevSetupCommand, DevAliasCommand } from "@/cypher/cli/dev-setup"
import { RemoteCommand } from "@/cli/cmd/remote"
import { ConfigCommand as ConfigCLICommand } from "@/cli/cmd/config"

const log = Log.create({ service: "cypher.cli" })

// All Cypher-specific CLI customization lives here so the shared upstream entrypoint
// (src/index.ts) only needs a handful of thin call-sites behind cypher_change markers.
// This keeps index.ts close to upstream and reduces merge conflicts on every sync.
export namespace CypherCli {
  // Register only the Cypher-specific commands. Upstream commands stay in index.ts's chain so
  // upstream merges that add or remove commands keep working without touching this file.
  export function register<T>(cli: Argv<T>): Argv<T> {
    cli
      .command(CypherConsoleCommand)
      .command(RollCallCommand)
      .command(ProfileCommand)
      .command(RemoteCommand)
      .command(DaemonCommand)
      .command(ConfigCLICommand)
    if (InstallationBuildKind !== "release") cli.command(DevSetupCommand).command(DevAliasCommand)
    // Safe self-reference: `cli` is a typed parameter and yargs `.command()` returns the same
    // instance, so the help command can resolve the fully-built root at handler time. This also
    // sidesteps the self-referential type error the old inline registration hit in index.ts.
    cli.command(createHelpCommand(() => cli))
    return cli
  }

  export async function runner() {
    if (!process.argv.includes("__background-process-runner")) return false
    return (await import("@/cypher/background-process/runner")).BackgroundProcessRunner.maybe()
  }

  // Runs from the upstream `.middleware`, before any command handler. Env tagging is additive so
  // it never has to modify upstream's own env assignments.
  export async function bootstrap(): Promise<void> {
    if (!process.env[ENV_FEATURE]) process.env[ENV_FEATURE] = process.argv.includes("serve") ? "unknown" : "cli"
    if (!process.env[ENV_VERSION]) process.env[ENV_VERSION] = InstallationVersion
    process.env.CYPHER = "1"

    const cfg = await AppRuntime.runPromise(Config.Service.use((c) => c.getGlobal()))
    await Telemetry.init({
      dataPath: Global.Path.data,
      version: InstallationVersion,
      enabled: cfg.experimental?.openTelemetry !== false,
    })

    // Migrate legacy Cypher CLI auth (~/.cypher/cli/config.json) into auth.json if present.
    await migrateLegacyCypherAuth(
      async () => (await AppRuntime.runPromise(Auth.Service.use((s) => s.get("cypher")))) !== undefined,
      async (auth) => AppRuntime.runPromise(Auth.Service.use((s) => s.set("cypher", auth))),
    )

    const auth = await AppRuntime.runPromise(Auth.Service.use((s) => s.get("cypher")))
    if (auth) {
      const token = auth.type === "oauth" ? auth.access : auth.key
      const account = auth.type === "oauth" ? auth.accountId : undefined
      await Telemetry.updateIdentity(token, account)
    }

    Telemetry.trackCliStart()
  }

  // Runs from the `finally` block on every exit path.
  export async function shutdown(): Promise<void> {
    const code = typeof process.exitCode === "number" ? process.exitCode : undefined
    Telemetry.trackCliExit(code)
    try {
      await SessionExport.shutdown()
      // Bound telemetry shutdown so an unreachable endpoint (offline, firewall,
      // DNS adblock resolving the host to 0.0.0.0) cannot block process exit on
      // short-lived commands like `cypher --help` / `cypher --version` (#9788).
      try {
        await Telemetry.shutdown(2000)
      } catch (err) {
        log.warn("telemetry shutdown failed", { err })
      }
    } finally {
      await CypherShutdown.run()
      await InstanceRuntime.disposeAllInstances() // safety net (no-op if already disposed)
    }
  }
}
