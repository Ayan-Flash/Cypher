package ai.cypher.client.actions

import ai.cypher.client.CypherNotifications
import ai.cypher.client.app.CypherWorkspaceService
import ai.cypher.client.plugin.CypherBundle
import ai.cypher.client.telemetry.Telemetry
import ai.cypher.rpc.dto.ConfigTargetDto
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.components.service
import com.intellij.openapi.project.DumbAware

abstract class ConfigAction(
    private val open: String,
    private val create: String,
    text: String,
    description: String,
) : AnAction(text, description, null), DumbAware {
    override fun getActionUpdateThread(): ActionUpdateThread = ActionUpdateThread.BGT

    protected fun text(target: ConfigTargetDto?): String {
        val key = if (target?.exists == false) create else open
        return CypherBundle.message(key, target?.displayPath ?: "...")
    }

    protected fun failed() {
        CypherNotifications.error(CypherBundle.message("action.Cypher.OpenConfig.failed"))
    }
}

class OpenLocalConfigAction : ConfigAction(
    open = "action.Cypher.OpenLocalConfig.text",
    create = "action.Cypher.CreateLocalConfig.text",
    text = CypherBundle.message("action.Cypher.OpenLocalConfig.text", "..."),
    description = CypherBundle.message("action.Cypher.OpenLocalConfig.description"),
) {
    override fun update(e: AnActionEvent) {
        val dir = e.workspaceDirectory()
        val service = service<CypherWorkspaceService>()
        val target = dir?.let { service.localConfig[it] }
        e.presentation.isEnabled = dir != null
        e.presentation.text = text(target)

        if (dir != null && target == null) {
            service.refreshLocalConfigTarget(dir)
        }
    }

    override fun actionPerformed(e: AnActionEvent) {
        val dir = e.workspaceDirectory() ?: return
        Telemetry.send("Config Opened", mapOf("surface" to "tool_window", "scope" to "local"))
        service<CypherWorkspaceService>().openLocalConfig(dir) { ok ->
            if (!ok) failed()
        }
    }
}

class OpenGlobalConfigAction : ConfigAction(
    open = "action.Cypher.OpenGlobalConfig.text",
    create = "action.Cypher.CreateGlobalConfig.text",
    text = CypherBundle.message("action.Cypher.OpenGlobalConfig.text", "..."),
    description = CypherBundle.message("action.Cypher.OpenGlobalConfig.description"),
) {
    override fun update(e: AnActionEvent) {
        val service = service<CypherWorkspaceService>()
        val target = service.globalConfig
        e.presentation.text = text(target)

        if (target == null) {
            service.refreshGlobalConfigTarget()
        }
    }

    override fun actionPerformed(e: AnActionEvent) {
        Telemetry.send("Config Opened", mapOf("surface" to "tool_window", "scope" to "global"))
        service<CypherWorkspaceService>().openGlobalConfig { ok ->
            if (!ok) failed()
        }
    }
}
