package ai.cypher.client.session

import ai.cypher.client.plugin.CypherBundle
import ai.cypher.client.ui.UiStyle

enum class SessionActivityKind {
    RUNNING,
    LOGIN_REQUIRED,
    PERMISSION,
    PLAN,
    QUESTION,
    ;

    fun label(): String = when (this) {
        RUNNING -> CypherBundle.message("session.part.tool.running")
        LOGIN_REQUIRED -> CypherBundle.message("history.badge.loginRequired")
        PERMISSION -> CypherBundle.message("history.badge.permission")
        PLAN -> CypherBundle.message("history.badge.plan")
        QUESTION -> CypherBundle.message("history.badge.question")
    }

    fun style(): UiStyle.Badge.Style = when (this) {
        RUNNING -> UiStyle.Badge.Alert
        LOGIN_REQUIRED, PERMISSION, PLAN, QUESTION -> UiStyle.Badge.Primary
    }
}
