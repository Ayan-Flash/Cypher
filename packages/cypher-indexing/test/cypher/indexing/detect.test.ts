import { describe, expect, test } from "bun:test"
import { mkdtemp } from "node:fs/promises"
import { tmpdir } from "node:os"
import { hasIndexingPlugin, isIndexingPlugin, normalizePluginName } from "../../../src/detect"

describe("indexing plugin detection", () => {
  test("bundles detect module for browser targets", async () => {
    const dir = await mkdtemp(`${tmpdir()}/cypher-indexing-detect-`)
    const result = await Bun.build({
      entrypoints: [new URL("../../../src/detect.ts", import.meta.url).pathname],
      minify: true,
      outdir: dir,
      target: "browser",
    })

    expect(result.success).toBe(true)
  })

  test("normalizes supported plugin forms", () => {
    expect(normalizePluginName("cypher-indexing")).toBe("cypher-indexing")
    expect(normalizePluginName("cypher-indexing@1.2.3")).toBe("cypher-indexing")
    expect(normalizePluginName("@cypher/cypher-indexing")).toBe("@cypher/cypher-indexing")
    expect(normalizePluginName("@cypher/cypher-indexing@1.2.3")).toBe("@cypher/cypher-indexing")
    expect(normalizePluginName("../../packages/cypher-indexing")).toBe("@cypher/cypher-indexing")
    expect(normalizePluginName("file:///tmp/.opencode/plugin/cypher-indexing.js")).toBe("cypher-indexing")
    expect(normalizePluginName("file:///tmp/node_modules/@cypher/cypher-indexing/index.js")).toBe(
      "@cypher/cypher-indexing",
    )
    expect(normalizePluginName("file:///tmp/repo/packages/cypher-indexing/src/index.ts")).toBe("@cypher/cypher-indexing")
  })

  test("detects supported indexing plugin specifiers", () => {
    const values = [
      "cypher-indexing",
      "cypher-indexing@1.2.3",
      "@cypher/cypher-indexing",
      "@cypher/cypher-indexing@1.2.3",
      "../../packages/cypher-indexing",
      "file:///tmp/.opencode/plugin/cypher-indexing.js",
      "file:///tmp/node_modules/@cypher/cypher-indexing/index.js",
      "file:///tmp/repo/packages/cypher-indexing/src/index.ts",
    ]

    for (const value of values) {
      expect(isIndexingPlugin(value)).toBe(true)
    }
  })

  test("ignores unrelated plugin specifiers", () => {
    expect(isIndexingPlugin("@cypher/cypher-gateway")).toBe(false)
    expect(isIndexingPlugin("file:///tmp/.opencode/plugin/index.js")).toBe(false)
    expect(hasIndexingPlugin(["@cypher/cypher-gateway", "foo@1.0.0"])).toBe(false)
  })

  test("detects indexing plugin in merged plugin lists", () => {
    expect(
      hasIndexingPlugin(["@cypher/cypher-gateway", "file:///tmp/node_modules/@cypher/cypher-indexing/index.js"]),
    ).toBe(true)
  })
})
