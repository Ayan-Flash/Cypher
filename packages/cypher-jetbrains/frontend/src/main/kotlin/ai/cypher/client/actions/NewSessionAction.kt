package ai.cypher.client.actions

import ai.cypher.client.plugin.CypherBundle
import ai.cypher.client.session.SessionManager
import ai.cypher.client.telemetry.Telemetry
import com.intellij.icons.AllIcons
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.project.DumbAware

class NewSessionAction : AnAction(
    CypherBundle.message("action.Cypher.NewSession.text"),
    CypherBundle.message("action.Cypher.NewSession.description"),
    AllIcons.General.Add,
), DumbAware {
    override fun actionPerformed(e: AnActionEvent) {
        Telemetry.send("New Session Clicked", mapOf("surface" to "tool_window"))
        e.getData(SessionManager.KEY)?.newSession()
    }

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabled = e.getData(SessionManager.KEY) != null
    }
}
