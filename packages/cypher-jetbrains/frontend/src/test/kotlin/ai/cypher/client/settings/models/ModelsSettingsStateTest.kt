package ai.cypher.client.settings.models

import ai.cypher.rpc.dto.AgentConfigDto
import ai.cypher.rpc.dto.AgentDto
import ai.cypher.rpc.dto.ConfigDto
import ai.cypher.rpc.dto.LoadErrorDto
import ai.cypher.rpc.dto.ProvidersDto
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ModelsSettingsStateTest {

    @Test
    fun `default model patch set and clear`() {
        val from = ModelsDraft(model = null)
        val set = ModelsDraft(model = "cypher/gpt-5")
        assertEquals("cypher/gpt-5", patch(from, set).values["model"])
        assertNull(patch(set, from).values["model"])
    }

    @Test
    fun `subagent clear clears variant`() {
        val from = ModelsDraft(subagent = "cypher/gpt-5", variant = "high")
        val to = ModelsDraft(subagent = null, variant = null)
        val patch = patch(from, to)
        assertNull(patch.values["subagent_model"])
        assertNull(patch.values["subagent_variant"])
    }

    @Test
    fun `per-mode patch set and clear`() {
        val from = ModelsDraft(agents = mapOf("ask" to null))
        val set = ModelsDraft(agents = mapOf("ask" to "openai/gpt"))
        assertEquals("openai/gpt", patch(from, set).agents["ask"]?.model)
        assertEquals(listOf("model"), patch(set, from).agents["ask"]?.clear)
        assertNull(patch(set, from).agents["ask"]?.model)
    }

    @Test
    fun `draft reads config agent values`() {
        val agents = listOf(AgentDto(name = "ask", displayName = "Ask", mode = "ask"))
        val config = ConfigDto(
            model = "cypher/gpt-5",
            smallModel = "cypher/auto-small",
            subagentModel = "openai/gpt",
            subagentVariant = "high",
            agent = mapOf("ask" to AgentConfigDto(model = "cypher/gpt-5")),
        )
        val draft = modelsDraft(config, agents)
        assertEquals("cypher/gpt-5", draft.model)
        assertEquals("cypher/auto-small", draft.small)
        assertEquals("openai/gpt", draft.subagent)
        assertEquals("high", draft.variant)
        assertEquals("cypher/gpt-5", draft.agents["ask"])
    }

    @Test
    fun `models status enables after providers load`() {
        val status = modelsStatus(
            ready = true,
            loading = false,
            providers = ProvidersDto(emptyList(), emptyList(), emptyMap()),
            items = 1,
            errors = emptyList(),
            saving = false,
        )

        assertEquals(ModelsStatus.READY, status)
    }

    @Test
    fun `models status allows default settings when agents fail`() {
        val status = modelsStatus(
            ready = true,
            loading = false,
            providers = ProvidersDto(emptyList(), emptyList(), emptyMap()),
            items = 1,
            errors = listOf(LoadErrorDto(resource = "agents", detail = "boom")),
            saving = false,
        )

        assertEquals(ModelsStatus.MODES_FAILED, status)
    }

    @Test
    fun `models status reports provider fetch failure`() {
        val status = modelsStatus(
            ready = true,
            loading = false,
            providers = null,
            items = 0,
            errors = listOf(LoadErrorDto(resource = "providers", detail = "boom")),
            saving = false,
        )

        assertEquals(ModelsStatus.LOAD_FAILED, status)
    }

    @Test
    fun `models status reports missing providers after loading completes`() {
        val status = modelsStatus(
            ready = true,
            loading = false,
            providers = null,
            items = 0,
            errors = emptyList(),
            saving = false,
        )

        assertEquals(ModelsStatus.LOAD_FAILED, status)
    }

    @Test
    fun `models status reports app unavailable`() {
        val status = modelsStatus(
            ready = false,
            loading = false,
            providers = null,
            items = 0,
            errors = emptyList(),
            saving = false,
        )

        assertEquals(ModelsStatus.UNAVAILABLE, status)
    }

    @Test
    fun `login banner shows only when ready and unauthenticated`() {
        assertTrue(modelsLoginBannerVisible(ready = true, authenticated = false))
        assertFalse(modelsLoginBannerVisible(ready = true, authenticated = true))
        assertFalse(modelsLoginBannerVisible(ready = false, authenticated = false))
    }

    @Test
    fun `saved match requires saved top level values`() {
        val draft = ModelsDraft(model = "cypher/gpt-5", small = "cypher/auto-small")

        assertTrue(savedMatches(draft, draft))
        assertFalse(savedMatches(draft.copy(model = "openai/gpt"), draft))
    }

    @Test
    fun `saved match compares known pending agent values`() {
        val draft = ModelsDraft(agents = mapOf("ask" to "cypher/gpt-5", "code" to null))
        val base = ModelsDraft(agents = mapOf("ask" to "cypher/gpt-5", "code" to null, "plan" to "openai/gpt"))

        assertTrue(savedMatches(base, draft))
        assertFalse(savedMatches(base.copy(agents = base.agents + ("ask" to "openai/gpt")), draft))
    }
}
