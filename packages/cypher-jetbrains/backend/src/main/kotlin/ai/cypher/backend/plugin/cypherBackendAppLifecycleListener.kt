package ai.cypher.backend.plugin

import ai.cypher.backend.app.CypherBackendAppService
import ai.cypher.log.CypherLog
import com.intellij.ide.AppLifecycleListener
import com.intellij.openapi.components.serviceIfCreated

class CypherBackendAppLifecycleListener : AppLifecycleListener {
    private val log = CypherLog.create(CypherBackendAppLifecycleListener::class.java)

    override fun appWillBeClosed(isRestart: Boolean) {
        log.info("appWillBeClosed(isRestart=$isRestart) — stopping Cypher CLI")
        runCatching {
            serviceIfCreated<CypherBackendAppService>()?.shutdownForAppClose()
        }.onFailure { log.warn("Failed to stop CLI on app close", it) }
    }
}
