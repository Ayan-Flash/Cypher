package ai.cypher.backend.plugin

import ai.cypher.CypherPlugin
import ai.cypher.backend.app.CypherBackendAppService
import ai.cypher.log.CypherLog
import com.intellij.ide.plugins.DynamicPluginListener
import com.intellij.ide.plugins.IdeaPluginDescriptor
import com.intellij.openapi.components.service

class CypherBackendDynamicPluginListener : DynamicPluginListener {
    private val log = CypherLog.create(CypherBackendDynamicPluginListener::class.java)

    override fun beforePluginUnload(pluginDescriptor: IdeaPluginDescriptor, isUpdate: Boolean) {
        if (pluginDescriptor.pluginId != CypherPlugin.id) return
        log.info("Shutting down Cypher backend for plugin unload (isUpdate=$isUpdate)")
        service<CypherBackendAppService>().shutdownForUnload()
    }
}
