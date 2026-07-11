import { describe, expect, test } from "bun:test"
import path from "path"

const root = path.join(__dirname, "..", "..")

describe("Cypher OAuth branding", () => {
  test("Codex OAuth browser flow uses Cypher branding", async () => {
    const src = await Bun.file(path.join(root, "src", "plugin", "openai", "codex.ts")).text()

    expect(src).toContain('originator: "cypher"')
    expect(src).toContain('"User-Agent": `cypher/${InstallationVersion}`')
    expect(src).toContain("return to Cypher")
    expect(src).not.toContain('originator: "opencode"')
    expect(src).not.toContain("return to OpenCode")
  })

  test("MCP OAuth callback page uses Cypher branding", async () => {
    const src = await Bun.file(path.join(root, "src", "mcp", "oauth-callback.ts")).text()

    expect(src).toContain("return to Cypher")
    expect(src).not.toContain("return to OpenCode")
  })
})
