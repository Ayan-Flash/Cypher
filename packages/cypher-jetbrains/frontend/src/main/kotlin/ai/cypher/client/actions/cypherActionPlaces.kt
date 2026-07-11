package ai.cypher.client.actions

import com.intellij.openapi.actionSystem.ActionPlaces

internal object CypherActionPlaces {
    const val CONNECTION_RETRY = "Cypher.ConnectionRetry"

    fun connectionRetryPopup() = ActionPlaces.getActionGroupPopupPlace(CONNECTION_RETRY)
}
