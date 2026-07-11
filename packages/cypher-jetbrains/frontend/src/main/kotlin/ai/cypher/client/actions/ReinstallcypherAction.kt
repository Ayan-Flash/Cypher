package ai.cypher.client.actions

import ai.cypher.client.app.CypherAppService
import ai.cypher.client.plugin.CypherBundle
import ai.cypher.client.telemetry.Telemetry
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.components.service
import com.intellij.openapi.project.DumbAware

class ReinstallCypherAction : AnAction(), DumbAware {
    override fun actionPerformed(e: AnActionEvent) {
        Telemetry.send("CLI Reinstall Clicked", mapOf("surface" to "settings"))
        service<CypherAppService>().reinstallAsync()
    }

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabled = true
        if (e.place == CypherActionPlaces.connectionRetryPopup()) {
            e.presentation.text = CypherBundle.message("action.Cypher.Reinstall.cli.text")
        }
    }
}
