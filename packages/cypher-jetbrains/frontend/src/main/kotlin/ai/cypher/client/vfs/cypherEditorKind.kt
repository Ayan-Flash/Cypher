package ai.cypher.client.vfs

import com.intellij.openapi.Disposable
import com.intellij.openapi.project.Project
import com.intellij.util.concurrency.annotations.RequiresEdt
import javax.swing.JComponent

interface CypherEditorKind : CypherVirtualFileKind {
    @RequiresEdt
    fun createContent(project: Project, file: CypherVirtualFile, parent: Disposable): JComponent

    fun preferredFocus(component: JComponent): JComponent? = null
}
