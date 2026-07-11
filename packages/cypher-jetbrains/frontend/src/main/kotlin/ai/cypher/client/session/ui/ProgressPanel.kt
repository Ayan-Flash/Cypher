package ai.cypher.client.session.ui

import ai.cypher.client.plugin.CypherBundle
import ai.cypher.client.session.model.SessionModel
import ai.cypher.client.session.model.SessionModelEvent
import ai.cypher.client.session.model.SessionState
import ai.cypher.client.session.ui.style.SessionEditorStyle
import ai.cypher.client.session.ui.style.SessionEditorStyleTarget
import ai.cypher.client.session.ui.style.SessionUiStyle
import ai.cypher.client.ui.UiStyle
import ai.cypher.client.ui.layout.Stack
import ai.cypher.client.ui.layout.StackAxis
import com.intellij.openapi.Disposable
import com.intellij.ui.AnimatedIcon
import com.intellij.ui.components.JBLabel
import com.intellij.util.ui.JBUI

/**
 * Progress footer rendered at the bottom of the session transcript while the
 * agent is working.
 *
 * Reacts to [SessionModelEvent.StateChanged]:
 * - [SessionState.Busy] → shows an animated spinner and [SessionState.Busy.text]
 * - [SessionState.Retry] → shows an animated spinner and retry detail
 * - [SessionState.Offline] → shows offline detail without a spinner
 * - Any other state -> hidden
 *
 * Owned by [SessionMessageListPanel], which always re-anchors it as the last child so it
 * appears below all turn views inside the scroll pane.
 */
class ProgressPanel(
    model: SessionModel,
    parent: Disposable,
) : Stack(StackAxis.HORIZONTAL, UiStyle.Gap.md()), SessionEditorStyleTarget {

    private var style = SessionEditorStyle.current()
    private var state: SessionState = SessionState.Idle
    private val label = JBLabel().apply {
        foreground = style.editorForeground
    }
    private val spinner = JBLabel(AnimatedIcon.Default())

    init {
        isOpaque = false
        isVisible = false
        border = JBUI.Borders.empty(
            UiStyle.Gap.sm(),
            JBUI.scale(SessionUiStyle.View.Layout.HORIZONTAL_PADDING),
            0,
            0,
        )
        applyStyle(SessionEditorStyle.current())

        next(spinner)
        next(label)

        model.addListener(parent) { event ->
            if (event is SessionModelEvent.StateChanged) onState(event.state)
        }
    }

    /** Exposed for test assertions. */
    fun labelText(): String = label.text

    /** Exposed for test assertions. */
    fun labelForeground() = label.foreground

    private fun onState(state: SessionState) {
        this.state = state
        when (state) {
            is SessionState.Busy -> {
                spinner.isVisible = true
                label.text = state.text
                label.foreground = style.editorForeground
                isVisible = true
            }
            is SessionState.Retry -> {
                spinner.isVisible = true
                label.text = retryText(state)
                label.foreground = UiStyle.Colors.warningLabelForeground()
                isVisible = true
            }
            is SessionState.Offline -> {
                spinner.isVisible = false
                label.text = state.message.ifBlank { CypherBundle.message("session.status.offline") }
                label.foreground = UiStyle.Colors.errorLabelForeground()
                isVisible = true
            }
            else -> isVisible = false
        }
        revalidate()
        repaint()
    }

    private fun retryText(state: SessionState.Retry): String {
        val base = state.message.ifBlank { CypherBundle.message("session.status.retry") }
        return if (state.attempt > 0) {
            CypherBundle.message("session.status.retry.attempt", base, state.attempt)
        } else base
    }

    override fun applyStyle(style: SessionEditorStyle) {
        this.style = style
        label.font = style.regularFont
        if (state is SessionState.Busy) label.foreground = style.editorForeground
        revalidate()
        repaint()
    }
}
