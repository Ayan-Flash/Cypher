package ai.cypher.client.settings.models

import ai.cypher.client.plugin.CypherBundle
import ai.cypher.client.settings.base.DraftReadyConfigurable
import com.intellij.openapi.project.ProjectManager
import kotlinx.coroutines.CoroutineScope
import javax.swing.JComponent

class ModelsConfigurable : DraftReadyConfigurable<JComponent>() {
    override fun getId(): String = ID

    override fun getDisplayName(): String = CypherBundle.message("settings.models.displayName")

    override fun create(cs: CoroutineScope): JComponent {
        val dir = ProjectManager.getInstance().openProjects.firstOrNull { !it.isDefault }?.basePath
        return ModelsSettingsUi(cs, directory = dir)
    }

    companion object {
        const val ID = "ai.cypher.jetbrains.settings.models"
    }
}
