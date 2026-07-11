package ai.cypher.backend.dev

import ai.cypher.log.CypherLog

object CypherDevMode {
    fun enabled(): Boolean = CypherLog.sandbox()
}
