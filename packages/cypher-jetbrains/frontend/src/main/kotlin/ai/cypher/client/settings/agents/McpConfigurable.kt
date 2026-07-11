package ai.cypher.client.settings.agents

import ai.cypher.client.app.CypherAgentBehaviorService
import ai.cypher.client.plugin.CypherBundle
import ai.cypher.client.settings.base.SettingsBadge
import ai.cypher.client.settings.base.SettingsListConfig
import ai.cypher.client.settings.base.SettingsListCell
import ai.cypher.client.settings.base.SettingsListItem
import ai.cypher.client.settings.base.SettingsListPanel
import ai.cypher.client.settings.base.SettingsListSelection
import ai.cypher.client.settings.base.SettingsMessageException
import ai.cypher.client.ui.UiStyle
import ai.cypher.client.ui.layout.Stack
import ai.cypher.log.CypherLog
import ai.cypher.rpc.dto.McpConfigDto
import ai.cypher.rpc.dto.McpServerConfigDto
import ai.cypher.rpc.dto.McpStatusDto
import com.intellij.icons.AllIcons
import com.intellij.openapi.application.EDT
import com.intellij.openapi.application.ModalityState
import com.intellij.openapi.application.asContextElement
import com.intellij.openapi.components.service
import com.intellij.openapi.ui.Messages
import com.intellij.ui.components.JBLabel
import com.intellij.util.ui.UIUtil
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.swing.JComponent

private val edt = Dispatchers.EDT + ModalityState.any().asContextElement()

class McpConfigurable : AgentBehaviorConfigurableBase<JComponent>() {
    override fun getId(): String = ID
    override fun getDisplayName(): String = CypherBundle.message("settings.agentBehavior.mcp.displayName")
    override fun create(cs: CoroutineScope, dir: String): JComponent = McpSettingsUi(cs, dir)
    override fun update(ui: JComponent, dir: String) {
        (ui as? McpSettingsUi)?.setDirectory(dir)
    }
    override fun scrollReadyShell() = false

    companion object { const val ID = "ai.cypher.jetbrains.settings.agentBehavior.mcp" }
}

internal class McpSettingsUi(
    cs: CoroutineScope,
    dir: String,
    private val create: (String, McpConfigDto) -> McpEditDialogHandle = ::McpEditDialog,
) : SettingsListPanel(cs, SettingsListConfig.Equal.copy(description = false)) {
    private var dir = dir

    private var servers: Map<String, McpServerConfigDto> = emptyMap()

    init {
        start()
    }

    fun setDirectory(value: String) {
        if (value == dir) return
        dir = value
        reload()
    }

    override suspend fun fetch(): List<SettingsListItem> {
        val behavior = service<CypherAgentBehaviorService>()
        val cfg = behavior.mcpConfig(dir)
        withContext(edt) { servers = cfg }
        val statuses = if (dir.isBlank()) {
            LOG.warn("mcp settings fetch skipped runtime status: missing project directory config=${cfg.size}")
            emptyMap()
        } else {
            behavior.mcpStatus(dir).associateBy { it.name }
        }
        val names = (cfg.keys + statuses.keys).sorted()
        LOG.info("mcp settings fetch dir=$dir config=${cfg.size} runtime=${statuses.size} total=${names.size}")
        if (names.isEmpty()) {
            LOG.warn("mcp settings fetch returned no servers dir=$dir")
        }
        return names.map { name -> item(name, cfg[name]?.config, statuses[name]) }
    }

    override fun onCell(key: String, cellId: String) {
        when (cellId) {
            CONNECT_CELL -> mutate(key) { service<CypherAgentBehaviorService>().mcpConnect(dir, key) }
            DISCONNECT_CELL -> mutate(key) { service<CypherAgentBehaviorService>().mcpDisconnect(dir, key) }
            AUTH_CELL -> mutate(key) { service<CypherAgentBehaviorService>().mcpAuthenticate(dir, key) }
            EDIT_CELL -> edit(key)
            REMOVE_CELL -> remove(key)
        }
    }

    override fun searchPlaceholder() = CypherBundle.message("settings.agentBehavior.mcp.search")

    override fun toolbarRight(): JComponent = Stack.horizontal(UiStyle.Gap.sm())
        .next(JBLabel(CypherBundle.message("settings.agentBehavior.mcp.addHint")).apply {
            foreground = UIUtil.getContextHelpForeground()
        })

    private fun item(name: String, cfg: McpConfigDto?, status: McpStatusDto?) = object : SettingsListItem {
        override val key = name
        override val title = name
        override val description = description(cfg, status)
        override val badges = badges(cfg, status)
        override val cells = cells(cfg, status)
    }

    private fun description(cfg: McpConfigDto?, status: McpStatusDto?): String? {
        val parts = listOfNotNull(
            cfg?.url?.takeIf { it.isNotBlank() },
            cfg?.command?.takeIf { it.isNotEmpty() }?.joinToString(" "),
            status?.error?.takeIf { it.isNotBlank() },
        )
        return parts.joinToString(" - ").takeIf { it.isNotBlank() }
    }

    private fun badges(cfg: McpConfigDto?, status: McpStatusDto?): List<SettingsBadge> = listOfNotNull(
        SettingsBadge(statusLabel(status), statusStyle(status)).takeIf { status != null },
        SettingsBadge(cfg?.type ?: CypherBundle.message("settings.agentBehavior.mcp.configured")).takeIf { cfg != null },
    )

    private fun cells(cfg: McpConfigDto?, status: McpStatusDto?): List<SettingsListCell> = listOfNotNull(
        connect(status?.status == CONNECTED),
        SettingsListCell(AUTH_CELL, CypherBundle.message("settings.agentBehavior.mcp.signIn")).takeIf {
            status?.status == NEEDS_AUTH
        },
        SettingsListCell(
            EDIT_CELL,
            CypherBundle.message("settings.agentBehavior.edit"),
            primary = true,
        ).takeIf { cfg != null },
        SettingsListCell(
            REMOVE_CELL,
            CypherBundle.message("common.delete"),
            icon = AllIcons.Actions.GC,
            iconOnly = true,
        ).takeIf { cfg != null },
    )

    private fun connect(connected: Boolean) = SettingsListCell(
        if (connected) DISCONNECT_CELL else CONNECT_CELL,
        if (connected) CypherBundle.message("settings.agentBehavior.mcp.disconnect")
        else CypherBundle.message("settings.agentBehavior.mcp.connect"),
    )

    private fun edit(name: String) {
        val server = servers[name] ?: return
        val dialog = create(name, server.config)
        if (!dialog.showAndGet()) return
        val next = dialog.result()
        mutateAndReload(selectionIndex()) {
            if (!service<CypherAgentBehaviorService>().saveMcp(dir, name, server.scope, next)) {
                throw SettingsMessageException(CypherBundle.message("settings.agentBehavior.save.failed"))
            }
            true
        }
    }

    private fun remove(name: String) {
        val result = Messages.showYesNoDialog(
            CypherBundle.message("settings.agentBehavior.mcp.delete.message", name),
            CypherBundle.message("settings.agentBehavior.mcp.delete.title"),
            CypherBundle.message("common.delete"),
            Messages.getCancelButton(),
            Messages.getQuestionIcon(),
        )
        if (result != Messages.YES) return
        val scope = servers[name]?.scope ?: return
        mutateAndReload(selectionIndex()) {
            if (!service<CypherAgentBehaviorService>().saveMcp(dir, name, scope, null)) {
                throw SettingsMessageException(CypherBundle.message("settings.agentBehavior.save.failed"))
            }
            true
        }
    }

    private fun mutate(name: String, block: suspend () -> Boolean) {
        mutateAndReload(SettingsListSelection.Key(name)) {
            if (!block()) throw SettingsMessageException(CypherBundle.message("settings.agentBehavior.mcp.action.failed"))
            true
        }
    }

    private companion object {
        const val CONNECTED = "connected"
        const val FAILED = "failed"
        const val NEEDS_AUTH = "needs_auth"
        const val NEEDS_REGISTRATION = "needs_client_registration"
        const val DISABLED = "disabled"
        const val CONNECT_CELL = "connect"
        const val DISCONNECT_CELL = "disconnect"
        const val AUTH_CELL = "auth"
        const val EDIT_CELL = "edit"
        const val REMOVE_CELL = "remove"
        val LOG = CypherLog.create(McpSettingsUi::class.java)

        fun statusLabel(status: McpStatusDto?): String {
            val value = status?.status ?: return ""
            return when (value) {
                CONNECTED -> CypherBundle.message("settings.agentBehavior.mcp.status.connected")
                FAILED -> CypherBundle.message("settings.agentBehavior.mcp.status.failed")
                NEEDS_AUTH -> CypherBundle.message("settings.agentBehavior.mcp.status.needsAuth")
                NEEDS_REGISTRATION -> CypherBundle.message("settings.agentBehavior.mcp.status.needsRegistration")
                DISABLED -> CypherBundle.message("settings.agentBehavior.mcp.status.disabled")
                else -> value
            }
        }

        fun statusStyle(status: McpStatusDto?): UiStyle.Badge.Style {
            return when (status?.status) {
                CONNECTED -> UiStyle.Badge.Highlight
                FAILED,
                NEEDS_AUTH,
                NEEDS_REGISTRATION,
                -> UiStyle.Badge.Alert
                else -> UiStyle.Badge.Secondary
            }
        }
    }
}
