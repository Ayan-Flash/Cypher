// cypher_change - new file
import { describe, expect, test } from "bun:test"
import { CypherRunAuto } from "../../src/cypher/cli/run-auto"

describe("CypherRunAuto", () => {
  test("tracks task child sessions without allowing unrelated sessions", () => {
    const state = CypherRunAuto.create("ses_root")

    expect(CypherRunAuto.allowed(state, "ses_root")).toBe(true)
    expect(CypherRunAuto.allowed(state, "ses_child")).toBe(false)

    CypherRunAuto.track(state, {
      type: "tool",
      tool: "task",
      sessionID: "ses_root",
      state: {
        metadata: {
          sessionId: "ses_child",
        },
      },
    })

    expect(CypherRunAuto.allowed(state, "ses_child")).toBe(true)
    expect(CypherRunAuto.allowed(state, "ses_other")).toBe(false)
  })

  test("ignores malformed or non-root task metadata", () => {
    const state = CypherRunAuto.create("ses_root")

    CypherRunAuto.track(state, {
      type: "tool",
      tool: "task",
      sessionID: "ses_root",
      state: {
        metadata: {
          sessionId: "",
        },
      },
    })
    CypherRunAuto.track(state, {
      type: "tool",
      tool: "task",
      sessionID: "ses_other",
      state: {
        metadata: {
          sessionId: "ses_wrong",
        },
      },
    })
    CypherRunAuto.track(state, {
      type: "text",
      sessionID: "ses_root",
      state: {},
    })

    expect(CypherRunAuto.allowed(state, "ses_wrong")).toBe(false)
    expect(CypherRunAuto.allowed(state, "")).toBe(false)
  })
})
