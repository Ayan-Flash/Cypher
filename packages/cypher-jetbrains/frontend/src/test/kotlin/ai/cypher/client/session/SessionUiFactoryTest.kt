package ai.cypher.client.session

import ai.cypher.client.app.CypherAppService
import ai.cypher.client.app.CypherSessionService
import ai.cypher.client.app.CypherWorkspaceService
import ai.cypher.client.app.Workspace
import ai.cypher.client.testing.FakeAppRpcApi
import ai.cypher.client.testing.FakeSessionRpcApi
import ai.cypher.client.testing.FakeWorkspaceRpcApi
import ai.cypher.rpc.dto.CypherAppStateDto
import ai.cypher.rpc.dto.CypherAppStatusDto
import ai.cypher.rpc.dto.CypherWorkspaceStateDto
import ai.cypher.rpc.dto.CypherWorkspaceStatusDto
import ai.cypher.rpc.dto.SessionDto
import ai.cypher.rpc.dto.SessionTimeDto
import com.intellij.openapi.components.service
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel

@Suppress("UnstableApiUsage")
class SessionUiFactoryTest : BasePlatformTestCase() {
    private lateinit var scope: CoroutineScope
    private lateinit var workspace: Workspace
    private lateinit var workspaces: CypherWorkspaceService
    private lateinit var sessions: CypherSessionService
    private lateinit var app: CypherAppService

    override fun setUp() {
        super.setUp()
        scope = CoroutineScope(SupervisorJob())
        sessions = CypherSessionService(project, scope, FakeSessionRpcApi())
        app = CypherAppService(scope, FakeAppRpcApi().also {
            it.state.value = CypherAppStateDto(CypherAppStatusDto.READY)
        })
        workspaces = CypherWorkspaceService(scope, FakeWorkspaceRpcApi().also {
            it.state.value = CypherWorkspaceStateDto(CypherWorkspaceStatusDto.READY)
        })
        workspace = workspaces.workspace("/test")
    }

    override fun tearDown() {
        try {
            scope.cancel()
        } finally {
            super.tearDown()
        }
    }

    fun `test factory creates blank session ui`() {
        val ui = direct().create(project, workspace, FakeManager(), null)

        assertNotNull(ui)
    }

    fun `test factory wires open callback`() {
        val manager = FakeManager()
        val rpc = session("ses_1")
        val ui = SessionUi(project, workspace, sessions, app, scope, manager = manager, workspaces = workspaces)
        val controller = controller(ui)

        com.intellij.openapi.application.ApplicationManager.getApplication().invokeAndWait {
            controller.openSession(rpc)
        }

        assertEquals(listOf("ses_1"), manager.opened)
    }

    fun `test empty panel opens through SessionRef via controller`() {
        val manager = FakeManager()
        val rpc = session("ses_1")
        val ui = SessionUi(project, workspace, sessions, app, scope, manager = manager, workspaces = workspaces)
        val controller = controller(ui)
        val panel = ai.cypher.client.session.ui.empty.EmptySessionPanel(testRootDisposable, controller, listOf(rpc))

        panel.clickRecent(0)

        // Recent click routes through SessionRef.Local path
        assertEquals(listOf("ses_1"), manager.opened)
    }

    fun `test empty panel show history routes through manager`() {
        val manager = FakeManager()
        val ui = SessionUi(project, workspace, sessions, app, scope, manager = manager, workspaces = workspaces)
        val controller = controller(ui)
        val panel = ai.cypher.client.session.ui.empty.EmptySessionPanel(
            testRootDisposable,
            controller,
            emptyList(),
            history = { manager.showHistory() },
        )

        panel.clickShowHistory()

        assertEquals(1, manager.history)
    }

    private fun controller(ui: SessionUi): ai.cypher.client.session.controller.SessionController {
        val field = SessionUi::class.java.getDeclaredField("controller")
        field.isAccessible = true
        return field.get(ui) as ai.cypher.client.session.controller.SessionController
    }

    fun `test application service is available`() {
        assertNotNull(service<SessionUiFactory>())
    }

    private fun direct() = SessionUiFactory(scope)

    private fun session(id: String) = SessionDto(
        id = id,
        projectID = "prj",
        directory = "/test",
        title = "Session $id",
        version = "1",
        time = SessionTimeDto(created = 1.0, updated = 2.0),
    )

    private class FakeManager : SessionManager {
        val opened = mutableListOf<String>()
        var history = 0
        override fun newSession() {
        }

        override fun showHistory() {
            history++
        }

        override fun openSession(ref: SessionRef) {
            val id = when (ref) {
                is SessionRef.Local -> ref.id
                is SessionRef.Cloud -> ref.key
            }
            opened.add(id)
        }
    }
}
