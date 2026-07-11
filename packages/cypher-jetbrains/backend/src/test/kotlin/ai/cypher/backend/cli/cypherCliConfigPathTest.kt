package ai.cypher.backend.cli

import kotlin.test.Test
import kotlin.test.assertEquals
import java.io.File
import java.nio.file.Files

class CypherCliConfigPathTest {

    @Test
    fun `cypher config dir overrides XDG config home`() {
        val dir = Files.createTempDirectory("cypher-config-dir").toFile()
        val xdg = Files.createTempDirectory("cypher-xdg-config").toFile()

        val path = CypherCliConfigPath.resolve(
            mapOf(
                "CYPHER_CONFIG_DIR" to dir.absolutePath,
                "XDG_CONFIG_HOME" to xdg.absolutePath,
            ),
        )

        assertEquals(dir.absoluteFile, path.absoluteFile)
    }

    @Test
    fun `XDG config home resolves to cypher subdirectory`() {
        val xdg = Files.createTempDirectory("cypher-xdg-config").toFile()

        val path = CypherCliConfigPath.resolve(mapOf("XDG_CONFIG_HOME" to xdg.absolutePath))

        assertEquals(File(xdg, "cypher").absoluteFile, path.absoluteFile)
    }

    @Test
    fun `default config home matches CLI xdg fallback`() {
        val home = Files.createTempDirectory("cypher-home").toFile()

        val path = CypherCliConfigPath.resolve(mapOf("HOME" to home.absolutePath))

        assertEquals(File(File(home, ".config"), "cypher").absoluteFile, path.absoluteFile)
    }

    @Test
    fun `USERPROFILE backs up HOME for default config home`() {
        val home = Files.createTempDirectory("cypher-userprofile").toFile()

        val path = CypherCliConfigPath.resolve(
            mapOf(
                "HOME" to "",
                "USERPROFILE" to home.absolutePath,
            ),
        )

        assertEquals(File(File(home, ".config"), "cypher").absoluteFile, path.absoluteFile)
    }

    @Test
    fun `blank config env values are ignored`() {
        val home = Files.createTempDirectory("cypher-home").toFile()

        val path = CypherCliConfigPath.resolve(
            mapOf(
                "CYPHER_CONFIG_DIR" to " ",
                "XDG_CONFIG_HOME" to "",
                "HOME" to home.absolutePath,
            ),
        )

        assertEquals(File(File(home, ".config"), "cypher").absoluteFile, path.absoluteFile)
    }

    @Test
    fun `legacy settings file resolves under global config dir`() {
        val home = Files.createTempDirectory("cypher-home").toFile()

        val path = CypherCliConfigPath.legacySettingsFile(mapOf("HOME" to home.absolutePath))

        assertEquals(File(File(File(home, ".config"), "cypher"), "legacy-settings.json").absoluteFile, path.absoluteFile)
    }
}
