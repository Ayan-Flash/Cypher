import { afterEach, beforeEach, describe, expect, mock, test } from "bun:test"

const calls: string[] = []
const timeouts: Array<number | undefined> = []
let err: unknown
let exit: string | number | null | undefined

mock.module("@opencode-ai/core/global", () => ({
  Global: { Path: { data: "/tmp/cypher-test" } },
}))

mock.module("@opencode-ai/core/installation/version", () => ({
  InstallationBuildKind: "release",
  InstallationVersion: "test",
}))

mock.module("@cypher/cypher-telemetry", () => ({
  Telemetry: {
    async init() {},
    async updateIdentity() {},
    trackCliStart() {},
    trackCliExit(code?: number) {
      calls.push(`track:${code ?? "undefined"}`)
    },
    async shutdown(timeout?: number) {
      calls.push("telemetry")
      timeouts.push(timeout)
      if (err) throw err
    },
  },
}))

mock.module("@cypher/cypher-gateway", () => ({
  ENV_FEATURE: "CYPHER_FEATURE",
  ENV_VERSION: "CYPHER_VERSION",
  async migrateLegacyCypherAuth() {},
}))

mock.module("@/config/config", () => ({
  Config: { Service: { use: () => ({ experimental: {} }) } },
}))

mock.module("@/auth", () => ({
  Auth: { Service: { use: () => undefined } },
}))

mock.module("@/project/instance-runtime", () => ({
  InstanceRuntime: {
    async disposeAllInstances() {
      calls.push("dispose")
    },
  },
}))

mock.module("@/cypher/session-export", () => ({
  SessionExport: {
    async shutdown() {
      calls.push("session")
    },
  },
}))

mock.module("@/cypher/help-command", () => ({
  createHelpCommand: () => ({ command: "help", handler() {} }),
}))

for (const path of [
  "@/cypher/cli/cmd/console",
  "@/cypher/cli/cmd/roll-call",
  "@/cypher/cli/cmd/profile",
  "@/cypher/cli/cmd/daemon",
  "@/cypher/cli/dev-setup",
  "@/cli/cmd/remote",
  "@/cli/cmd/config",
]) {
  mock.module(path, () => ({
    CypherConsoleCommand: { command: "console", handler() {} },
    RollCallCommand: { command: "roll-call", handler() {} },
    ProfileCommand: { command: "profile", handler() {} },
    DaemonCommand: { command: "daemon", handler() {} },
    DevSetupCommand: { command: "dev-setup", handler() {} },
    DevAliasCommand: { command: "dev-alias", handler() {} },
    RemoteCommand: { command: "remote", handler() {} },
    ConfigCommand: { command: "config", handler() {} },
  }))
}

describe("CypherCli.shutdown", () => {
  beforeEach(() => {
    calls.length = 0
    timeouts.length = 0
    err = undefined
    exit = process.exitCode
    process.exitCode = undefined
  })

  afterEach(() => {
    process.exitCode = exit
  })

  test("keeps telemetry shutdown timeout best-effort and still disposes instances", async () => {
    err = "Timeout while shutting down PostHog. Some events may not have been sent."
    process.exitCode = 0
    const { CypherCli } = await import("../../src/cypher/cli/setup")

    await expect(CypherCli.shutdown()).resolves.toBeUndefined()

    expect(timeouts).toEqual([2000])
    expect(calls).toEqual(["track:0", "session", "telemetry", "dispose"])
    expect(process.exitCode).toBe(0)
  })

  test("preserves failing command exit status", async () => {
    process.exitCode = 1
    const { CypherCli } = await import("../../src/cypher/cli/setup")

    await CypherCli.shutdown()

    expect(timeouts).toEqual([2000])
    expect(calls).toEqual(["track:1", "session", "telemetry", "dispose"])
    expect(process.exitCode).toBe(1)
  })
})
