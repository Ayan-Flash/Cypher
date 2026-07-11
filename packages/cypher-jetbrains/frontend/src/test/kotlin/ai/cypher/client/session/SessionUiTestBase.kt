package ai.cypher.client.session

import ai.cypher.client.app.CypherAppService
import ai.cypher.client.app.CypherSessionService
import ai.cypher.client.app.CypherWorkspaceService
import ai.cypher.client.app.Workspace
import ai.cypher.client.migration.FakeMigrationUiController
import ai.cypher.client.migration.MigrationUiController
import ai.cypher.client.session.ui.SessionRootPanel
import ai.cypher.client.session.ui.prompt.PromptPanel
import ai.cypher.client.session.controller.SessionController
import ai.cypher.client.testing.FakeAppRpcApi
import ai.cypher.client.testing.FakeSessionRpcApi
import ai.cypher.client.testing.FakeWorkspaceRpcApi
import ai.cypher.client.testing.TestCoroutines
import ai.cypher.client.session.SessionRef
import ai.cypher.client.session.scroll.SessionScroll
import ai.cypher.rpc.dto.ChatEventDto
import ai.cypher.rpc.dto.CypherAppStateDto
import ai.cypher.rpc.dto.CypherAppStatusDto
import ai.cypher.rpc.dto.CypherWorkspaceStateDto
import ai.cypher.rpc.dto.CypherWorkspaceStatusDto
import ai.cypher.rpc.dto.MessageDto
import ai.cypher.rpc.dto.MessageTimeDto
import ai.cypher.rpc.dto.MessageWithPartsDto
import ai.cypher.rpc.dto.PartDto
import ai.cypher.rpc.dto.SessionDto
import ai.cypher.rpc.dto.SessionTimeDto
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import com.intellij.openapi.util.Disposer
import com.intellij.util.ui.UIUtil
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import java.awt.Container
import java.awt.event.MouseEvent
import java.awt.event.MouseWheelEvent
import javax.swing.JLabel
import javax.swing.JComponent
import javax.swing.JScrollBar

@Suppress("UnstableApiUsage")
abstract class SessionUiTestBase : BasePlatformTestCase() {
    private lateinit var coroutines: TestCoroutines
    protected lateinit var scope: CoroutineScope
    protected lateinit var sessions: CypherSessionService
    protected lateinit var app: CypherAppService
    protected lateinit var workspaces: CypherWorkspaceService
    protected lateinit var rpc: FakeSessionRpcApi
    protected lateinit var appRpc: FakeAppRpcApi
    protected lateinit var workspace: Workspace
    protected lateinit var ui: SessionUi

    override fun setUp() {
        super.setUp()
        coroutines = TestCoroutines()
        scope = coroutines.scope

        rpc = FakeSessionRpcApi()
        appRpc = FakeAppRpcApi().also {
            it.state.value = CypherAppStateDto(CypherAppStatusDto.READY)
        }
        val workspaceRpc = FakeWorkspaceRpcApi().also {
            it.state.value = CypherWorkspaceStateDto(status = CypherWorkspaceStatusDto.READY)
        }

        sessions = CypherSessionService(project, scope, rpc)
        app = CypherAppService(scope, appRpc)
        workspaces = CypherWorkspaceService(scope, workspaceRpc)
        workspace = workspaces.workspace("/test")

        ui = newUi()
        layout()
    }

    override fun tearDown() {
        try {
            Disposer.dispose(ui)
            coroutines.close { UIUtil.dispatchAllInvocationEvents() }
        } finally {
            super.tearDown()
        }
    }

    protected fun newUi(
        id: String? = null,
        displayMs: Long = 0,
        open: ((SessionRef) -> Unit)? = null,
        migration: MigrationUiController = FakeMigrationUiController(),
    ): SessionUi {
        val manager = open?.let { fn ->
            object : SessionManager {
                override fun newSession() {}
                override fun showHistory() {}
                override fun openSession(ref: SessionRef) = fn(ref)
            }
        }
        return SessionUi(
            project, workspace, sessions, app, scope,
            ref = SessionRef.from(id),
            displayMs = displayMs,
            manager = manager,
            workspaces = workspaces,
            migration = migration,
        ).apply {
            setSize(800, 600)
        }
    }

    protected fun layout() {
        ui.doLayout()
        val root = find<SessionRootPanel>(ui)
        root.doLayout()
        root.content.doLayout()
        find<PromptPanel>(ui).parent.doLayout()
        scrollComponent().doLayout()
        (scrollView() as? Container)?.doLayout()
    }

    protected fun settle() {
        coroutines.drain { UIUtil.dispatchAllInvocationEvents() }
    }

    protected fun settleShort(ms: Long) = runBlocking {
        delay(ms)
        UIUtil.dispatchAllInvocationEvents()
    }

    protected fun showMessages() {
        controller().prompt("hello")
        settle()
        layout()
    }

    protected fun fillTranscript(count: Int, start: Int = 0) {
        repeat(count) { offset ->
            val i = start + offset
            val id = "msg_$i"
            emit(ChatEventDto.MessageUpdated("ses_test", message(id)), flush = false)
            emit(ChatEventDto.PartUpdated("ses_test", part("part_$i", id, "text", text(i))), flush = false)
        }
        settle()
        forceFlush()
        drainScroll()
    }

    protected fun emit(event: ChatEventDto, flush: Boolean = true) {
        runBlocking { rpc.events.emit(event) }
        if (flush) {
            settle()
            forceFlush()
        }
    }

    protected fun forceFlush() {
        controller().flushEvents()
        UIUtil.dispatchAllInvocationEvents()
    }

    protected fun forceFlushWithoutDispatch() {
        controller().flushEvents()
    }

    protected fun drainScroll() {
        repeat(4) {
            layout()
            UIUtil.dispatchAllInvocationEvents()
        }
    }

    private fun scroll(): SessionScroll = ui.scroll

    protected fun scrollComponent(): JComponent = scroll().component

    protected fun scrollView(): JComponent? = scroll().view

    protected fun scrollBar(): JScrollBar = scroll().bar

    protected fun jumpButton(): JLabel = scroll().jump

    protected fun click(label: JLabel) {
        val event = MouseEvent(label, MouseEvent.MOUSE_CLICKED, System.currentTimeMillis(), 0, 1, 1, 1, false)
        for (listener in label.mouseListeners) listener.mouseClicked(event)
    }

    protected fun bottom(bar: JScrollBar): Int = (bar.maximum - bar.visibleAmount).coerceAtLeast(0)

    protected fun setBottom(bar: JScrollBar) {
        setValue(bar, bottom(bar))
    }

    protected fun setValue(bar: JScrollBar, value: Int) {
        wheelNoop()
        setValuePassive(bar, value)
    }

    protected fun setValuePassive(bar: JScrollBar, value: Int) {
        bar.value = value.coerceIn(bar.minimum, bottom(bar))
    }

    protected fun wheelNoop() {
        val event = MouseWheelEvent(scrollComponent(), MouseEvent.MOUSE_WHEEL, System.currentTimeMillis(), 0, 1, 1, 0, false, MouseWheelEvent.WHEEL_UNIT_SCROLL, 1, 1)
        for (listener in scrollComponent().mouseWheelListeners) listener.mouseWheelMoved(event)
    }

    protected fun assertBottom(bar: JScrollBar) {
        assertTrue("value=${bar.value} bottom=${bottom(bar)} max=${bar.maximum} visible=${bar.visibleAmount}", bar.value >= bottom(bar) - 1)
    }

    protected inline fun <reified T> find(root: Container): T {
        return find(root, T::class.java) ?: error("missing ${T::class.java.simpleName}")
    }

    protected fun <T> find(root: Container, cls: Class<T>): T? {
        if (cls.isInstance(root)) return cls.cast(root)
        for (child in root.components) {
            if (cls.isInstance(child)) return cls.cast(child)
            if (child is Container) {
                val item = find(child, cls)
                if (item != null) return item
            }
        }
        return null
    }

    protected fun controller(): SessionController {
        val field = SessionUi::class.java.getDeclaredField("controller")
        field.isAccessible = true
        return field.get(ui) as SessionController
    }

    protected fun session(id: String) = SessionDto(
        id = id,
        projectID = "prj",
        directory = "/test",
        title = "Recent $id",
        version = "1",
        time = SessionTimeDto(created = 1.0, updated = 2.0),
    )

    protected fun message(id: String) = MessageDto(
        id = id,
        sessionID = "ses_test",
        role = "user",
        time = MessageTimeDto(created = 0.0),
    )

    protected fun part(id: String, mid: String, type: String, text: String? = null) = PartDto(
        id = id,
        sessionID = "ses_test",
        messageID = mid,
        type = type,
        text = text,
    )

    protected fun history(count: Int): List<MessageWithPartsDto> = List(count) { i ->
        val id = "hist_$i"
        MessageWithPartsDto(message(id), listOf(part("hist_part_$i", id, "text", text(i))))
    }

    protected fun text(i: Int): String = "line $i\n".repeat(12)
}
