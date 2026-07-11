import { describe, expect, test } from "bun:test"
import { CypherPtySelfCommand } from "../../src/cypher/pty/self-command"

describe("pty self-command", () => {
  test("does not forward bundled bun entrypoints", () => {
    const proc = {
      argv: ["/tmp/cypher", "/$bunfs/root/src/index.js"],
      execArgv: ["--user-agent=cypher/test", "--use-system-ca", "--"],
      execPath: "/tmp/cypher",
      cwd: "/tmp",
    }

    const cmd = CypherPtySelfCommand.command(proc)
    expect(cmd).toStrictEqual({ command: "/tmp/cypher", args: [] })
    expect(CypherPtySelfCommand.resolve({ command: "cypher", cwd: "/tmp/project" }, cmd)).toStrictEqual({
      command: "/tmp/cypher",
      args: [],
      cwd: "/tmp/project",
    })
    expect(
      CypherPtySelfCommand.command({
        ...proc,
        argv: ["C:/tmp/cypher.exe", "B:/~BUN/root/src/index.js"],
      }).args,
    ).toStrictEqual([])
    expect(
      CypherPtySelfCommand.command({
        ...proc,
        argv: ["C:/tmp/cypher.exe", "b:\\~BUN\\root\\src\\index.js"],
      }).args,
    ).toStrictEqual([])
  })

  test("forwards source entrypoints", () => {
    const cmd = CypherPtySelfCommand.command({
      argv: ["/tmp/bun", "/tmp/cypher/src/index.ts"],
      execArgv: ["--conditions=browser", "--cwd", "packages/opencode"],
      execPath: "/tmp/bun",
      cwd: "/tmp/cypher",
    })
    expect(cmd).toStrictEqual({
      command: "/tmp/bun",
      args: ["--conditions=browser", "/tmp/cypher/src/index.ts"],
      cwd: "/tmp/cypher",
    })
    expect(CypherPtySelfCommand.resolve({ command: "cypher", cwd: "/tmp/project" }, cmd)).toStrictEqual({
      command: "/tmp/bun",
      args: ["--conditions=browser", "/tmp/cypher/src/index.ts", "/tmp/project"],
      cwd: "/tmp/cypher",
    })
  })
})
