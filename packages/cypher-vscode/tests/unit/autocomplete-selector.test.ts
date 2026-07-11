import { describe, expect, it } from "bun:test"
import { notebookModel, selector } from "../../src/services/autocomplete/AutocompleteServiceManager"

describe("autocomplete document selector", () => {
  it("registers classic autocomplete for files and notebook cells", () => {
    expect(selector("classic")).toEqual([{ scheme: "file" }, { scheme: "vscode-notebook-cell" }])
  })

  it("keeps Next Edit limited to files", () => {
    expect(selector("next-edit")).toEqual([{ scheme: "file" }])
  })

  it("uses the matching FIM model for notebook fallback", () => {
    expect(notebookModel("cypher", "inception/mercury-next-edit").id).toBe("cypher/inception/mercury-edit-2")
    expect(notebookModel("inception", "mercury-next-edit").id).toBe("inception/mercury-edit-2")
    expect(notebookModel("cypher", "mistralai/codestral-2508").id).toBe("cypher/mistralai/codestral-2508")
  })
})
