package ai.cypher.client.session.controller

import ai.cypher.rpc.dto.ConfigWarningDto
import ai.cypher.rpc.dto.CypherWorkspaceStatusDto
import ai.cypher.rpc.dto.CypherAppStateDto
import ai.cypher.rpc.dto.CypherAppStatusDto

class AppWatchingTest : SessionControllerTestBase() {

    fun `test app state change fires AppChanged`() {
        val m = controller()
        val events = collect(m)
        flush()
        events.clear()

        appRpc.state.value = CypherAppStateDto(CypherAppStatusDto.READY)
        flush()

        assertControllerEvents("AppChanged", events)
        assertSession(
            """
            [app: READY] [workspace: PENDING]
            """,
            m,
            show = false,
        )
    }

    fun `test retry connection uses app retry when app is failed`() {
        val m = controller()
        val events = collect(m)
        appRpc.state.value = CypherAppStateDto(CypherAppStatusDto.ERROR, error = "boom")

        flush()
        events.clear()
        edt { m.retryConnection() }
        flush()

        assertEquals(1, appRpc.retries)
        assertEquals(0, projectRpc.reloads)
        assertTrue(events.any { it is SessionControllerEvent.ConnectionChanged.ShowConnecting })
    }

    fun `test retry connection reloads workspace when app ready and workspace failed`() {
        val m = controller()
        val events = collect(m)
        appRpc.state.value = CypherAppStateDto(CypherAppStatusDto.READY)
        projectRpc.state.value = ai.cypher.rpc.dto.CypherWorkspaceStateDto(
            status = CypherWorkspaceStatusDto.ERROR,
            error = "workspace fail",
        )

        flush()
        events.clear()
        edt { m.retryConnection() }
        flush()

        assertEquals(0, appRpc.retries)
        assertEquals(1, projectRpc.reloads)
        assertTrue(events.any { it is SessionControllerEvent.ConnectionChanged.ShowConnecting })
    }

    fun `test retry connection uses app retry when app has warnings`() {
        val m = controller()
        val events = collect(m)
        appRpc.state.value = CypherAppStateDto(
            status = CypherAppStatusDto.READY,
            warnings = listOf(ConfigWarningDto(path = ".cypher/cypher.json", message = "Invalid JSON")),
        )

        flush()
        events.clear()
        edt { m.retryConnection() }
        flush()

        assertEquals(1, appRpc.retries)
        assertEquals(0, projectRpc.reloads)
        assertTrue(events.any { it is SessionControllerEvent.ConnectionChanged.ShowConnecting })
    }

    fun `test retry connection immediately updates connection state`() {
        val m = controller()
        val states = collectStates(m)
        appRpc.state.value = CypherAppStateDto(CypherAppStatusDto.ERROR, error = "boom")
        flush()
        states.clear()

        edt { m.retryConnection() }
        flush()

        val state = states.single { it.first is SessionControllerEvent.ConnectionChanged.ShowConnecting }.second
        assertEquals(SessionControllerEvent.ConnectionChanged.ShowConnecting, state.connectionState)
        assertEquals(SessionControllerEvent.ConnectionChanged.ShowConnecting, state.connectionTargetState)
    }
}
