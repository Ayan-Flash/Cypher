package ai.cypher.backend.cli

import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import java.io.File
import java.nio.file.Files

class CypherBackendCliManagerEnvTest {

    private lateinit var tmp: File
    private val manager = CypherBackendCliManager()

    @BeforeTest
    fun setUp() {
        tmp = Files.createTempDirectory("cypher-cli-env-test").toFile()
        System.clearProperty("cypher.dev.storage.isolated")
        System.clearProperty("cypher.dev.worktree.root")
        System.clearProperty("idea.plugin.in.sandbox.mode")
    }

    @AfterTest
    fun tearDown() {
        CypherClaudeCompatSettings.set(false)
        System.clearProperty("cypher.dev.storage.isolated")
        System.clearProperty("cypher.dev.worktree.root")
        System.clearProperty("idea.plugin.in.sandbox.mode")
        tmp.deleteRecursively()
    }

    @Test
    fun `isolation disabled - required JetBrains env vars are present`() {
        val env = manager.buildEnv("pwd123", emptyMap())

        assertEquals("jetbrains", env["CYPHER_CLIENT"])
        assertEquals("true", env["CYPHER_ENABLE_QUESTION_TOOL"])
        assertEquals("jetbrains", env["CYPHER_PLATFORM"])
        assertEquals("cypher-cli", env["CYPHER_APP_NAME"])
        assertEquals("all", env["CYPHER_TELEMETRY_LEVEL"])
        assertEquals("true", env["CYPHER_DISABLE_CLAUDE_CODE"])
        assertEquals("jetbrains-plugin", env["CYPHER_FEATURE"])
        assertEquals("pwd123", env["CYPHER_SERVER_PASSWORD"])
    }

    @Test
    fun `dev mode disables CLI telemetry`() {
        System.setProperty("idea.plugin.in.sandbox.mode", "true")

        val env = manager.buildEnv("pwd123", emptyMap())

        assertEquals("off", env["CYPHER_TELEMETRY_LEVEL"])
    }

    @Test
    fun `claude compatibility omits disable env var`() {
        CypherClaudeCompatSettings.set(true)

        val env = manager.buildEnv("pwd123", emptyMap())

        assertFalse(env.containsKey("CYPHER_DISABLE_CLAUDE_CODE"))
    }

    @Test
    fun `isolation disabled - default CLI config asks for edit and bash permissions`() {
        val env = manager.buildEnv("pwd123", emptyMap())

        assertEquals("""{"permission":{"edit":"ask","bash":"ask"}}""", env["CYPHER_CONFIG_CONTENT"])
    }

    @Test
    fun `isolation disabled - base CLI config is preserved`() {
        val cfg = """{"permission":{"edit":"allow"}}"""

        val env = manager.buildEnv("pwd123", mapOf("CYPHER_CONFIG_CONTENT" to cfg))

        assertEquals(cfg, env["CYPHER_CONFIG_CONTENT"])
    }

    @Test
    fun `isolation disabled - base PATH is preserved`() {
        val path = "/opt/homebrew/bin:/usr/bin"

        val env = manager.buildEnv("pwd123", mapOf("PATH" to path))

        assertEquals(path, env["PATH"])
    }

    @Test
    fun `isolation disabled - no XDG storage overrides are injected`() {
        val env = manager.buildEnv("pwd123", emptyMap())

        assertFalse(env.containsKey("XDG_DATA_HOME"), "XDG_DATA_HOME should not be set when isolation is off")
        assertFalse(env.containsKey("XDG_CONFIG_HOME"), "XDG_CONFIG_HOME should not be set when isolation is off")
        assertFalse(env.containsKey("XDG_STATE_HOME"), "XDG_STATE_HOME should not be set when isolation is off")
        assertFalse(env.containsKey("XDG_CACHE_HOME"), "XDG_CACHE_HOME should not be set when isolation is off")
    }

    @Test
    fun `isolation enabled - XDG vars point under cypher-dev in worktree root`() {
        System.setProperty("cypher.dev.storage.isolated", "true")
        System.setProperty("cypher.dev.worktree.root", tmp.absolutePath)

        val env = manager.buildEnv("pwd123", emptyMap())

        val dev = File(tmp, ".cypher-dev")
        assertEquals(File(dev, "data").absolutePath, env["XDG_DATA_HOME"])
        assertEquals(File(dev, "config").absolutePath, env["XDG_CONFIG_HOME"])
        assertEquals(File(dev, "state").absolutePath, env["XDG_STATE_HOME"])
        assertEquals(File(dev, "cache").absolutePath, env["XDG_CACHE_HOME"])
    }

    @Test
    fun `isolation enabled - cypher-dev subdirectories are created`() {
        System.setProperty("cypher.dev.storage.isolated", "true")
        System.setProperty("cypher.dev.worktree.root", tmp.absolutePath)

        manager.buildEnv("pwd123", emptyMap())

        val dev = File(tmp, ".cypher-dev")
        assertTrue(File(dev, "data").isDirectory, "data dir should be created")
        assertTrue(File(dev, "config").isDirectory, "config dir should be created")
        assertTrue(File(dev, "state").isDirectory, "state dir should be created")
        assertTrue(File(dev, "cache").isDirectory, "cache dir should be created")
    }

    @Test
    fun `isolation enabled - required JetBrains env vars are still present`() {
        System.setProperty("cypher.dev.storage.isolated", "true")
        System.setProperty("cypher.dev.worktree.root", tmp.absolutePath)

        val env = manager.buildEnv("pwd123", emptyMap())

        assertEquals("jetbrains", env["CYPHER_CLIENT"])
        assertEquals("pwd123", env["CYPHER_SERVER_PASSWORD"])
        assertEquals("jetbrains-plugin", env["CYPHER_FEATURE"])
    }

    @Test
    fun `isolation enabled - base env vars are preserved`() {
        System.setProperty("cypher.dev.storage.isolated", "true")
        System.setProperty("cypher.dev.worktree.root", tmp.absolutePath)

        val env = manager.buildEnv("pwd123", mapOf("MY_CUSTOM_VAR" to "hello"))

        assertEquals("hello", env["MY_CUSTOM_VAR"])
    }

    @Test
    fun `isolation enabled without worktree root - no XDG vars are injected`() {
        System.setProperty("cypher.dev.storage.isolated", "true")
        // cypher.dev.worktree.root is intentionally not set

        val env = manager.buildEnv("pwd123", emptyMap())

        assertFalse(env.containsKey("XDG_DATA_HOME"), "XDG_DATA_HOME should not be set when root is missing")
        assertFalse(env.containsKey("XDG_CONFIG_HOME"), "XDG_CONFIG_HOME should not be set when root is missing")
        assertFalse(env.containsKey("XDG_STATE_HOME"), "XDG_STATE_HOME should not be set when root is missing")
        assertFalse(env.containsKey("XDG_CACHE_HOME"), "XDG_CACHE_HOME should not be set when root is missing")
    }
}
