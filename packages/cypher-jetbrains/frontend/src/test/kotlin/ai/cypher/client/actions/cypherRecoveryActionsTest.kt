package ai.cypher.client.actions

import ai.cypher.client.app.CypherAppService
import ai.cypher.client.app.CypherWorkspaceService
import ai.cypher.client.app.Workspace
import ai.cypher.client.session.SessionManager
import ai.cypher.client.testing.FakeAppRpcApi
import ai.cypher.client.testing.FakeWorkspaceRpcApi
import ai.cypher.rpc.dto.ConfigTargetDto
import ai.cypher.rpc.dto.CypherWorkspaceStateDto
import ai.cypher.rpc.dto.CypherWorkspaceStatusDto
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.DataContext
import com.intellij.openapi.actionSystem.DefaultActionGroup
import com.intellij.openapi.actionSystem.Presentation
import com.intellij.openapi.actionSystem.ex.ActionUtil
import com.intellij.openapi.application.ApplicationManager
import com.intellij.testFramework.replaceService
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout

@Suppress("UnstableApiUsage")
class CypherRecoveryActionsTest : BasePlatformTestCase() {
    private lateinit var scope: CoroutineScope
    private lateinit var rpc: FakeWorkspaceRpcApi
    private lateinit var appRpc: FakeAppRpcApi

    override fun setUp() {
        super.setUp()
        scope = CoroutineScope(SupervisorJob())
        rpc = FakeWorkspaceRpcApi()
        appRpc = FakeAppRpcApi()
        ApplicationManager.getApplication().replaceService(
            CypherAppService::class.java,
            CypherAppService(scope, appRpc),
            testRootDisposable,
        )
        ApplicationManager.getApplication().replaceService(
            CypherWorkspaceService::class.java,
            CypherWorkspaceService(scope, rpc),
            testRootDisposable,
        )
    }

    override fun tearDown() {
        try {
            scope.cancel()
        } finally {
            super.tearDown()
        }
    }

    fun `test restart action stays enabled for all app states`() {
        val action = RestartCypherAction()
        val event = event(action)

        update(action, event)

        assertTrue("Restart should force-enable recovery action", event.presentation.isEnabled)
    }

    fun `test reinstall action stays enabled for all app states`() {
        val action = ReinstallCypherAction()
        val event = event(action)

        update(action, event)

        assertTrue("Reinstall should force-enable recovery action", event.presentation.isEnabled)
    }

    fun `test restart action adds core suffix in connection retry popup`() {
        val action = RestartCypherAction()
        val event = event(action, place = CypherActionPlaces.connectionRetryPopup())

        update(action, event)

        assertEquals("Restart Core", event.presentation.text)
    }

    fun `test reinstall action adds core suffix in connection retry popup`() {
        val action = ReinstallCypherAction()
        val event = event(action, place = CypherActionPlaces.connectionRetryPopup())

        update(action, event)

        assertEquals("Reinstall Core", event.presentation.text)
    }

    fun `test core group has visible menu text and info action`() {
        val xml = requireNotNull(javaClass.classLoader.getResourceAsStream("cypher.jetbrains.frontend.xml"))
            .bufferedReader()
            .use { it.readText() }

        assertTrue(xml.contains("<group id=\"Cypher.CliGroup\" text=\"Core\" popup=\"true\">"))
        assertTrue(xml.contains("<reference ref=\"Cypher.Restart\"/>"))
        assertTrue(xml.contains("<reference ref=\"Cypher.Reinstall\"/>"))
        assertTrue(xml.contains("<reference ref=\"Cypher.CoreInfo\"/>"))
        assertTrue(xml.contains("<group id=\"Cypher.OpenConfigGroup\" text=\"Config Files\" popup=\"true\">"))
        assertTrue(xml.contains("<reference ref=\"Cypher.OpenConfigGroup\"/>"))
        assertFalse(xml.contains("<action id=\"Cypher.ShowProfile\""))
        assertFalse(xml.contains("<reference ref=\"Cypher.ShowProfile\"/>"))
    }

    fun `test core info action shows version and architecture`() {
        appRpc.cliVersion = "1.2.3"
        appRpc.cliPlatform = "darwin-arm64"
        ApplicationManager.getApplication().executeOnPooledThread {
            runBlocking { app().coreInfo() }
        }.get()
        val action = CoreInfoAction()
        val event = event(action)

        update(action, event)

        assertFalse(event.presentation.isEnabled)
        assertTrue(event.presentation.isVisible)
        assertEquals("Core v1.2.3 • Architecture: darwin-arm64", event.presentation.text)
    }

    fun `test local config action says open when target exists`() {
        rpc.localConfigPath = "/test/.cypher/cypher.jsonc"
        rpc.localConfigDisplayPath = "~/.cypher/cypher.jsonc"
        rpc.localConfigExists = true
        service().localConfig["/test"] = ConfigTargetDto("/test/.cypher/cypher.jsonc", "~/.cypher/cypher.jsonc", true)
        val action = OpenLocalConfigAction()
        val event = event(action, workspace = workspace("/test"))

        update(action, event)

        assertTrue(event.presentation.isEnabled)
        assertEquals("Open: local ~/.cypher/cypher.jsonc", event.presentation.text)
        assertEquals(0, rpc.localConfigPathCalls)
    }

    fun `test local config action says create when target is missing`() {
        rpc.localConfigPath = "/test/.cypher/cypher.jsonc"
        rpc.localConfigDisplayPath = "~/.cypher/cypher.jsonc"
        rpc.localConfigExists = false
        service().localConfig["/test"] = ConfigTargetDto("/test/.cypher/cypher.jsonc", "~/.cypher/cypher.jsonc", false)
        val action = OpenLocalConfigAction()
        val event = event(action, workspace = workspace("/test"))

        update(action, event)

        assertTrue(event.presentation.isEnabled)
        assertEquals("Create: local ~/.cypher/cypher.jsonc", event.presentation.text)
        assertEquals(0, rpc.localConfigPathCalls)
    }

    fun `test local config action refreshes missing target in background`() {
        rpc.localConfigPath = "/test/.cypher/cypher.jsonc"
        rpc.localConfigDisplayPath = "/test/.cypher/cypher.jsonc"
        rpc.localConfigExists = true
        val call = CompletableDeferred<Unit>()
        val gate = CompletableDeferred<Unit>()
        rpc.beforeLocalConfigTarget = {
            call.complete(Unit)
            gate.await()
        }
        val action = OpenLocalConfigAction()
        val event = event(action, workspace = workspace("/test"))

        update(action, event)

        assertTrue(event.presentation.isEnabled)
        assertEquals("Open: local ...", event.presentation.text)
        await(call)
        assertEquals(1, rpc.localConfigPathCalls)

        gate.complete(Unit)
        service().localConfig["/test"] = ConfigTargetDto("/test/.cypher/cypher.jsonc", "/test/.cypher/cypher.jsonc", true)

        val next = event(action, workspace = workspace("/test"))
        update(action, next)

        assertEquals("Open: local /test/.cypher/cypher.jsonc", next.presentation.text)
    }

    fun `test local config action dedupes in flight refresh`() {
        val gate = CompletableDeferred<Unit>()
        val call = CompletableDeferred<Unit>()
        val action = OpenLocalConfigAction()
        rpc.beforeLocalConfigTarget = {
            call.complete(Unit)
            gate.await()
        }

        update(action, event(action, workspace = workspace("/test")))
        await(call)
        update(action, event(action, workspace = workspace("/test")))

        assertEquals(1, rpc.localConfigPathCalls)

        gate.complete(Unit)
    }

    fun `test global config action says open when target exists`() {
        rpc.globalConfigPath = "/config/cypher.jsonc"
        rpc.globalConfigDisplayPath = "~/.config/cypher/cypher.jsonc"
        rpc.globalConfigExists = true
        cacheGlobal(ConfigTargetDto("/config/cypher.jsonc", "~/.config/cypher/cypher.jsonc", true))
        val action = OpenGlobalConfigAction()
        val event = event(action)

        update(action, event)

        assertEquals("Open: global ~/.config/cypher/cypher.jsonc", event.presentation.text)
        assertEquals(0, rpc.globalConfigPathCalls)
    }

    fun `test global config action says create when target is missing`() {
        rpc.globalConfigPath = "/config/cypher.jsonc"
        rpc.globalConfigDisplayPath = "~/.config/cypher/cypher.jsonc"
        rpc.globalConfigExists = false
        cacheGlobal(ConfigTargetDto("/config/cypher.jsonc", "~/.config/cypher/cypher.jsonc", false))
        val action = OpenGlobalConfigAction()
        val event = event(action)

        update(action, event)

        assertEquals("Create: global ~/.config/cypher/cypher.jsonc", event.presentation.text)
        assertEquals(0, rpc.globalConfigPathCalls)
    }

    fun `test global config action refreshes missing target in background`() {
        rpc.globalConfigPath = "/config/cypher.jsonc"
        rpc.globalConfigDisplayPath = "/config/cypher.jsonc"
        rpc.globalConfigExists = true
        val call = CompletableDeferred<Unit>()
        val gate = CompletableDeferred<Unit>()
        rpc.beforeGlobalConfigTarget = {
            call.complete(Unit)
            gate.await()
        }
        val action = OpenGlobalConfigAction()
        val event = event(action)

        update(action, event)

        assertEquals("Open: global ...", event.presentation.text)
        await(call)
        assertEquals(1, rpc.globalConfigPathCalls)

        gate.complete(Unit)
        cacheGlobal(ConfigTargetDto("/config/cypher.jsonc", "/config/cypher.jsonc", true))

        val next = event(action)
        update(action, next)

        assertEquals("Open: global /config/cypher.jsonc", next.presentation.text)
    }

    fun `test global config action dedupes in flight refresh`() {
        val gate = CompletableDeferred<Unit>()
        val call = CompletableDeferred<Unit>()
        rpc.beforeGlobalConfigTarget = {
            call.complete(Unit)
            gate.await()
        }
        val action = OpenGlobalConfigAction()

        update(action, event(action))
        await(call)
        update(action, event(action))

        assertEquals(1, rpc.globalConfigPathCalls)

        gate.complete(Unit)
    }

    fun `test local config action disables without directory`() {
        val action = OpenLocalConfigAction()
        val event = event(action)

        update(action, event)

        assertFalse(event.presentation.isEnabled)
        assertEquals(0, rpc.localConfigPathCalls)
    }

    fun `test settings popup group updates recursively in background`() {
        val group = DefaultActionGroup()
        val wrapped = CypherSettingsAction.popupGroup(group)

        assertEquals(ActionUpdateThread.BGT, wrapped.actionUpdateThread)
    }

    fun `test settings action prewarms config targets`() {
        val action = CypherSettingsAction()

        runBlocking {
            CypherSettingsAction.refreshConfigTargets(event(action, workspace = workspace("/test")), service()).forEach { it.join() }
        }

        assertEquals(1, rpc.localConfigPathCalls)
        assertEquals(1, rpc.globalConfigPathCalls)
    }

    fun `test workspace creation prewarms config targets`() {
        val local = CompletableDeferred<Unit>()
        val global = CompletableDeferred<Unit>()
        rpc.beforeLocalConfigTarget = { local.complete(Unit) }
        rpc.beforeGlobalConfigTarget = { global.complete(Unit) }

        service().workspace("/test")

        await(local)
        await(global)
        assertEquals(1, rpc.localConfigPathCalls)
        assertEquals(1, rpc.globalConfigPathCalls)
    }

    private fun event(action: AnAction, workspace: Workspace? = null, place: String = ""): AnActionEvent {
        val presentation = Presentation().apply { copyFrom(action.templatePresentation) }
        presentation.isEnabled = false
        return AnActionEvent.createFromDataContext(place, presentation, context(workspace))
    }

    private fun update(action: AnAction, event: AnActionEvent) {
        ApplicationManager.getApplication().executeOnPooledThread {
            ActionUtil.updateAction(action, event)
        }.get()
    }

    private fun await(signal: CompletableDeferred<Unit>) = runBlocking {
        withTimeout(5_000) { signal.await() }
    }

    private fun service(): CypherWorkspaceService = ApplicationManager.getApplication().getService(CypherWorkspaceService::class.java)

    private fun app(): CypherAppService = ApplicationManager.getApplication().getService(CypherAppService::class.java)

    private fun cacheGlobal(target: ConfigTargetDto) {
        val field = CypherWorkspaceService::class.java.getDeclaredField("globalConfig")
        field.isAccessible = true
        field.set(service(), target)
    }

    private fun context(workspace: Workspace?): DataContext {
        return DataContext { id ->
            when (id) {
                SessionManager.WORKSPACE_KEY.name -> workspace
                CommonDataKeys.PROJECT.name -> project.takeIf { workspace != null }
                else -> null
            }
        }
    }

    private fun workspace(dir: String): Workspace {
        return Workspace(
            dir,
            MutableStateFlow(CypherWorkspaceStateDto(CypherWorkspaceStatusDto.READY)),
            reload = {},
        )
    }
}
