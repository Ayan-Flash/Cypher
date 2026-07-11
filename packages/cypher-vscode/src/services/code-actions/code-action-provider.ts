import * as vscode from "vscode"

export class CypherActionProvider implements vscode.CodeActionProvider {
  static readonly metadata: vscode.CodeActionProviderMetadata = {
    providedCodeActionKinds: [vscode.CodeActionKind.QuickFix, vscode.CodeActionKind.RefactorRewrite],
  }

  provideCodeActions(
    document: vscode.TextDocument,
    range: vscode.Range | vscode.Selection,
    context: vscode.CodeActionContext,
  ): vscode.CodeAction[] {
    if (range.isEmpty) return []

    const actions: vscode.CodeAction[] = []

    const add = new vscode.CodeAction("Add to Cypher CLI", vscode.CodeActionKind.RefactorRewrite)
    add.command = { command: "cypher-cli.new.addToContext", title: "Add to Cypher CLI" }
    actions.push(add)

    const hasDiagnostics = context.diagnostics.length > 0

    if (hasDiagnostics) {
      const fix = new vscode.CodeAction("Fix with Cypher CLI", vscode.CodeActionKind.QuickFix)
      fix.command = { command: "cypher-cli.new.fixCode", title: "Fix with Cypher CLI" }
      fix.isPreferred = true
      actions.push(fix)
    }

    if (!hasDiagnostics) {
      const explain = new vscode.CodeAction("Explain with Cypher CLI", vscode.CodeActionKind.RefactorRewrite)
      explain.command = { command: "cypher-cli.new.explainCode", title: "Explain with Cypher CLI" }
      actions.push(explain)

      const improve = new vscode.CodeAction("Improve with Cypher CLI", vscode.CodeActionKind.RefactorRewrite)
      improve.command = { command: "cypher-cli.new.improveCode", title: "Improve with Cypher CLI" }
      actions.push(improve)
    }

    return actions
  }
}
