package ai.cypher.backend.cli

import java.util.Properties

object CypherProps {
    private val props by lazy {
        val stream = CypherProps::class.java.classLoader.getResourceAsStream("cypher.properties")
            ?: throw IllegalStateException("cypher.properties resource not found")
        stream.use {
            Properties().apply { load(it) }
        }
    }

    fun cliVersion(): String = props.getProperty("cli.version")
        ?: throw IllegalStateException("cli.version missing from cypher.properties")

    fun pinned(): Boolean = pinned(props)

    internal fun pinned(props: Properties): Boolean = props.getProperty("cli.pinned")?.toBoolean() ?: true
}
