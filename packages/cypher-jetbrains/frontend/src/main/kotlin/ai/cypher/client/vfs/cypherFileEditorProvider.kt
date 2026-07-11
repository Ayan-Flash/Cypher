package ai.cypher.client.vfs

import ai.cypher.client.session.ui.attachment.ensureAttachmentEditorKind
import com.intellij.openapi.components.service
import com.intellij.openapi.fileEditor.FileEditor
import com.intellij.openapi.fileEditor.FileEditorPolicy
import com.intellij.openapi.fileEditor.FileEditorProvider
import com.intellij.openapi.project.DumbAware
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.vfs.VirtualFile

class CypherFileEditorProvider : FileEditorProvider, DumbAware {
    override fun accept(project: Project, file: VirtualFile): Boolean {
        ensureAttachmentEditorKind()
        val path = path(file) ?: return false
        return service<CypherEditorKindRegistry>().get(path.kind) != null
    }

    override fun acceptRequiresReadAction(): Boolean = false

    override fun createEditor(project: Project, file: VirtualFile): FileEditor {
        ensureAttachmentEditorKind()
        val path = path(file) ?: error("Invalid Cypher virtual file: ${file.path}")
        val cypher = file as? CypherVirtualFile ?: CypherVirtualFile(path)
        val kind = service<CypherEditorKindRegistry>().get(cypher.path.kind) ?: error("Unknown Cypher editor kind: ${cypher.path.kind}")
        return CypherFileEditor(project, file, cypher, kind)
    }

    override fun disposeEditor(editor: FileEditor) {
        Disposer.dispose(editor)
    }

    override fun getEditorTypeId(): String = EDITOR_TYPE_ID
    override fun getPolicy(): FileEditorPolicy = FileEditorPolicy.HIDE_OTHER_EDITORS

    companion object {
        const val EDITOR_TYPE_ID = "CypherVfsEditor"

        private fun path(file: VirtualFile): CypherPath? {
            if (file is CypherVirtualFile) return file.path
            if (file.fileSystem.protocol != CypherVirtualFileSystem.PROTOCOL && !file.url.startsWith("${CypherVirtualFileSystem.PROTOCOL}://")) return null
            return CypherVirtualFileSystem.decode(file.path) ?: CypherVirtualFileSystem.decode(file.url)
        }
    }
}
