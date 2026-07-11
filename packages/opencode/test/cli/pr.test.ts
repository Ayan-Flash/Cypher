// cypher_change - new file
import { expect, test } from "bun:test"
import { cliCommand } from "../../src/cli/cmd/pr"

test("cliCommand uses the current script when argv[1] is a file path", () => {
  const result = cliCommand({
    execPath: "/usr/bin/node",
    argv: ["/usr/bin/node", "/tmp/cypher.js", "pr", "1"],
    exists: (file) => file === "/tmp/cypher.js",
  })

  expect(result).toEqual(["/usr/bin/node", "/tmp/cypher.js"])
})

test("cliCommand falls back to execPath when argv[1] is a subcommand", () => {
  const result = cliCommand({
    execPath: "/usr/local/bin/cypher",
    argv: ["/usr/local/bin/cypher", "pr", "1"],
    exists: () => false,
  })

  expect(result).toEqual(["/usr/local/bin/cypher"])
})

test("cliCommand ignores subcommand token even when it exists on disk", () => {
  const result = cliCommand({
    execPath: "/usr/local/bin/cypher",
    argv: ["/usr/local/bin/cypher", "pr", "1"],
    exists: (file) => file === "pr",
  })

  expect(result).toEqual(["/usr/local/bin/cypher"])
})

test("cliCommand falls back to execPath when argv[1] is missing", () => {
  const result = cliCommand({
    execPath: "/usr/local/bin/cypher",
    argv: ["/usr/local/bin/cypher"],
    exists: () => false,
  })

  expect(result).toEqual(["/usr/local/bin/cypher"])
})

test("cliCommand falls back to execPath for bun virtual script paths", () => {
  const unix = cliCommand({
    execPath: "/tmp/cypher",
    argv: ["/tmp/cypher", "/$bunfs/root/src/index.js", "pr", "1"],
    exists: () => true,
  })

  const win = cliCommand({
    execPath: "C:/tmp/cypher.exe",
    argv: ["C:/tmp/cypher.exe", "B:/~BUN/root/src/index.js", "pr", "1"],
    exists: () => true,
  })

  expect(unix).toEqual(["/tmp/cypher"])
  expect(win).toEqual(["C:/tmp/cypher.exe"])
})
