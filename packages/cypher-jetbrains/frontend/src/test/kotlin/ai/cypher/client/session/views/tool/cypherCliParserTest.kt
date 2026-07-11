package ai.cypher.client.session.views.tool

import ai.cypher.cli.CypherCliParser
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class CypherCliParserTest {
    @Test
    fun `tag extracts trimmed tool xml value`() {
        val text = """
            <path>
              /tmp/example.txt
            </path>
            <type>file</type>
        """.trimIndent()

        assertEquals("/tmp/example.txt", CypherCliParser.tag(text, "path"))
        assertEquals("file", CypherCliParser.tag(text, "type"))
    }

    @Test
    fun `tag returns null for blank or missing value`() {
        assertNull(CypherCliParser.tag("<path>   </path>", "path"))
        assertNull(CypherCliParser.tag("<type>file</type>", "path"))
    }
}
