package ai.cypher.client.session.ui

import ai.cypher.client.plugin.CypherBundle
import ai.cypher.client.session.model.SessionState
import ai.cypher.client.session.ui.style.SessionEditorStyle
import ai.cypher.client.session.ui.style.SessionEditorStyleTarget
import ai.cypher.client.ui.UiStyle
import com.intellij.ui.components.JBLabel
import com.intellij.util.ui.Centerizer
import java.awt.BorderLayout
import javax.swing.JPanel

class LoadingPanel : JPanel(BorderLayout()), SessionEditorStyleTarget {
    private val label = JBLabel(CypherBundle.message("session.empty.loading"))

    init {
        isOpaque = false
        add(Centerizer(label, Centerizer.TYPE.BOTH), BorderLayout.CENTER)
        applyStyle(SessionEditorStyle.current())
    }

    fun setState(state: SessionState) {
        when (state) {
            is SessionState.Retry -> {
                label.text = state.message.ifBlank { CypherBundle.message("session.status.retry") }
                label.foreground = UiStyle.Colors.warningLabelForeground()
            }

            is SessionState.Offline -> {
                label.text = state.message.ifBlank { CypherBundle.message("session.status.offline") }
                label.foreground = UiStyle.Colors.errorLabelForeground()
            }

            else -> {
                label.text = CypherBundle.message("session.empty.loading")
                label.foreground = UiStyle.Colors.weak()
            }
        }
        revalidate()
        repaint()
    }

    /** Exposed for test assertions. */
    fun labelText(): String = label.text

    override fun applyStyle(style: SessionEditorStyle) {
        label.font = style.regularFont
        revalidate()
        repaint()
    }
}
