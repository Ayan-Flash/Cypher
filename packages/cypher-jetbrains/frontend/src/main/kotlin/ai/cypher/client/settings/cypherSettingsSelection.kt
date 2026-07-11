package ai.cypher.client.settings

import ai.cypher.client.settings.profile.UserProfileConfigurable
import com.intellij.ide.util.PropertiesComponent
import com.intellij.openapi.project.Project

internal object CypherSettingsSelection {
    // IntelliJ persists the selected settings page with SettingsEditor.SELECTED_CONFIGURABLE.
    const val SELECTED_CONFIGURABLE_KEY = "settings.editor.selected.configurable"

    fun target(project: Project): String {
        val id = PropertiesComponent.getInstance(project).getValue(SELECTED_CONFIGURABLE_KEY)
        if (id != null && isCypher(id)) return id
        return UserProfileConfigurable.ID
    }

    private fun isCypher(id: String?): Boolean {
        if (id == CypherSettingsConfigurable.ID) return true
        return id?.startsWith("${CypherSettingsConfigurable.ID}.") == true
    }
}
