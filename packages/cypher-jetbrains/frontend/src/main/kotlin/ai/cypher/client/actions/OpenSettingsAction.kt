package ai.cypher.client.actions

import ai.cypher.client.plugin.CypherBundle
import ai.cypher.client.settings.CypherSettingsConfigurable
import ai.cypher.client.settings.CypherSettingsSelection
import ai.cypher.client.telemetry.Telemetry
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.options.Configurable
import com.intellij.openapi.options.ConfigurableWithId
import com.intellij.openapi.options.ShowSettingsUtil
import com.intellij.openapi.project.DumbAwareAction
import com.intellij.openapi.project.ProjectManager
import java.util.function.Predicate

class OpenSettingsAction : DumbAwareAction(
    CypherBundle.message("action.Cypher.OpenSettings.text"),
    CypherBundle.message("action.Cypher.OpenSettings.description"),
    null,
) {
    override fun actionPerformed(e: AnActionEvent) {
        Telemetry.send("Settings Opened", mapOf("surface" to "tool_window"))
        val project = e.project ?: ProjectManager.getInstance().defaultProject
        val target = CypherSettingsSelection.target(project)
        val util = ShowSettingsUtil.getInstance()
        try {
            util.showSettingsDialog(project, predicate(target), null)
        } catch (err: IllegalStateException) {
            if (target == CypherSettingsConfigurable.ID) throw err
            util.showSettingsDialog(project, predicate(CypherSettingsConfigurable.ID), null)
        }
    }

    override fun getActionUpdateThread(): ActionUpdateThread = ActionUpdateThread.BGT

    private fun predicate(id: String) = Predicate { cfg: Configurable ->
        cfg is ConfigurableWithId && cfg.getId() == id
    }
}
