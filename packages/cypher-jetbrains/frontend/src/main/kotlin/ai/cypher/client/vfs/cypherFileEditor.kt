package ai.cypher.client.vfs

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.util.concurrency.annotations.RequiresEdt
import javax.swing.JComponent

class CypherFileEditor(
    private val project: Project,
    private val file: VirtualFile,
    private val cypher: CypherVirtualFile,
    private val kind: CypherEditorKind,
) : CypherFileEditorBase() {
    private val ui: JComponent by lazy { kind.createContent(project, cypher, this) }

    @RequiresEdt
    override fun getComponent(): JComponent = ui

    override fun getPreferredFocusedComponent(): JComponent? = kind.preferredFocus(ui)
    override fun getName(): String = kind.title(cypher.path.params)
    override fun getFile(): VirtualFile = file
    override fun isValid(): Boolean = super.isValid() && cypher.isValid

    override fun dispose() {
        CypherVirtualFileSystem.getInstance().release(cypher.path)
        super.dispose()
    }
}
