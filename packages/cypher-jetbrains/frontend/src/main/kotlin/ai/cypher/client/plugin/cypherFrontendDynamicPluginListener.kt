package ai.cypher.client.plugin

import ai.cypher.CypherPlugin
import ai.cypher.client.session.ui.attachment.unregisterAttachmentEditorKind
import ai.cypher.client.vfs.CypherEditorKindRegistry
import ai.cypher.client.vfs.CypherVirtualFileSystem
import ai.cypher.log.CypherLog
import com.intellij.ide.plugins.DynamicPluginListener
import com.intellij.ide.plugins.IdeaPluginDescriptor
import com.intellij.openapi.components.service
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.project.ProjectManager
import com.intellij.openapi.wm.ToolWindowManager
import javax.swing.SwingUtilities

class CypherFrontendDynamicPluginListener : DynamicPluginListener {
    override fun beforePluginUnload(pluginDescriptor: IdeaPluginDescriptor, isUpdate: Boolean) {
        if (pluginDescriptor.pluginId != CypherPlugin.id) return
        CypherFrontendUnloadCleanup.cleanup(isUpdate)
    }
}

object CypherFrontendUnloadCleanup {
    private val log = CypherLog.create(CypherFrontendUnloadCleanup::class.java)

    fun cleanup(isUpdate: Boolean) {
        log.info("Cleaning up Cypher frontend for plugin unload (isUpdate=$isUpdate)")
        runEdt {
            ProjectManager.getInstance().openProjects.forEach { project ->
                if (project.isDisposed) return@forEach
                ToolWindowManager.getInstance(project).getToolWindow("Cypher CLI")
                    ?.contentManager
                    ?.removeAllContents(true)
                val editors = FileEditorManager.getInstance(project).openFiles
                    .filter { it.fileSystem === CypherVirtualFileSystem.getInstance() }
                editors.forEach { file -> FileEditorManager.getInstance(project).closeFile(file) }
            }
        }
        unregisterAttachmentEditorKind()
        service<CypherEditorKindRegistry>().clear()
        CypherVirtualFileSystem.getInstance().clear()
    }

    private fun runEdt(block: () -> Unit) {
        if (SwingUtilities.isEventDispatchThread()) {
            block()
            return
        }
        SwingUtilities.invokeAndWait(block)
    }
}
