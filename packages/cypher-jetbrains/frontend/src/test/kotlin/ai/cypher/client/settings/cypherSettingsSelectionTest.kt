package ai.cypher.client.settings

import ai.cypher.client.settings.models.ModelsConfigurable
import ai.cypher.client.settings.profile.UserProfileConfigurable
import com.intellij.ide.util.PropertiesComponent
import com.intellij.testFramework.fixtures.BasePlatformTestCase

class CypherSettingsSelectionTest : BasePlatformTestCase() {

    override fun tearDown() {
        try {
            PropertiesComponent.getInstance(project).unsetValue(CypherSettingsSelection.SELECTED_CONFIGURABLE_KEY)
        } finally {
            super.tearDown()
        }
    }

    fun `test falls back to profile when no last settings page exists`() {
        assertEquals(UserProfileConfigurable.ID, CypherSettingsSelection.target(project))
    }

    fun `test falls back to profile when last page is not cypher`() {
        select("preferences.lookFeel")

        assertEquals(UserProfileConfigurable.ID, CypherSettingsSelection.target(project))
    }

    fun `test keeps last cypher root page`() {
        select(CypherSettingsConfigurable.ID)

        assertEquals(CypherSettingsConfigurable.ID, CypherSettingsSelection.target(project))
    }

    fun `test keeps last cypher child page`() {
        select(ModelsConfigurable.ID)

        assertEquals(ModelsConfigurable.ID, CypherSettingsSelection.target(project))
    }

    private fun select(id: String) {
        PropertiesComponent.getInstance(project).setValue(CypherSettingsSelection.SELECTED_CONFIGURABLE_KEY, id)
    }
}
