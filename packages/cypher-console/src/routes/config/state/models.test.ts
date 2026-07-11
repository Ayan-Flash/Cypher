import { describe, expect, test } from "bun:test"
import { hasGateway, visible } from "./privacy"

describe("model privacy filter", () => {
  test("detects when Cypher Gateway models are present", () => {
    expect(hasGateway([{ id: "cypher" }, { id: "openai" }])).toBe(true)
    expect(hasGateway([{ id: "openai" }])).toBe(false)
  })

  test("shows every model when disabled", () => {
    expect(visible({ id: "cypher" }, { mayTrainOnYourPrompts: true }, false)).toBe(true)
  })

  test("hides only Cypher Gateway models explicitly marked for prompt training", () => {
    expect(visible({ id: "cypher" }, { mayTrainOnYourPrompts: true }, true)).toBe(false)
    expect(visible({ id: "cypher" }, { mayTrainOnYourPrompts: false }, true)).toBe(true)
    expect(visible({ id: "cypher" }, {}, true)).toBe(true)
    expect(visible({ id: "openai" }, { mayTrainOnYourPrompts: true }, true)).toBe(true)
  })
})
