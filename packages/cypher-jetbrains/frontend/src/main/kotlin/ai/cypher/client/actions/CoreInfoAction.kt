package ai.cypher.client.actions

import ai.cypher.client.app.CypherAppService
import ai.cypher.client.plugin.CypherBundle
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.components.service
import com.intellij.openapi.project.DumbAware

class CoreInfoAction : AnAction(), DumbAware {
    override fun actionPerformed(e: AnActionEvent) = Unit

    override fun update(e: AnActionEvent) {
        val app = service<CypherAppService>()
        val info = app.core
        if (info == null) app.fetchCoreInfoAsync()
        e.presentation.text = info?.let {
            CypherBundle.message("action.Cypher.CoreInfo.text", it.version, it.platform)
        } ?: CypherBundle.message("action.Cypher.CoreInfo.loading")
        e.presentation.description = CypherBundle.message("action.Cypher.CoreInfo.description")
        e.presentation.isEnabled = false
        e.presentation.isVisible = true
    }

    override fun getActionUpdateThread(): ActionUpdateThread = ActionUpdateThread.BGT
}
