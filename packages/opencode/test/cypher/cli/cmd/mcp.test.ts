import { describe, expect, test } from "bun:test"
import { CyphercodeMcpConfig } from "@/cypher/cli/cmd/mcp"

const added = `{
  "permission": {
    "bash": "allow"
  },
  "mcp": {
    "linear": {
      "type": "remote",
      "url": "https://mcp.linear.app/mcp",
      "oauth": {}
    }
  },
}`

describe("CyphercodeMcpConfig.format", () => {
  test("writes strict JSON for cypher.json", () => {
    const output = CyphercodeMcpConfig.format("/tmp/cypher.json", added)

    expect(JSON.parse(output)).toEqual({
      permission: { bash: "allow" },
      mcp: {
        linear: {
          type: "remote",
          url: "https://mcp.linear.app/mcp",
          oauth: {},
        },
      },
    })
    expect(output).not.toEndWith(",\n}")
  })

  test("preserves JSONC formatting for cypher.jsonc", () => {
    expect(CyphercodeMcpConfig.format("/tmp/cypher.jsonc", added)).toBe(added)
  })
})
