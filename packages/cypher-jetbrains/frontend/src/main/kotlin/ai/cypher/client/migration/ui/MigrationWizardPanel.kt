package ai.cypher.client.migration.ui

import ai.cypher.client.migration.MigrationItemUiProgress
import ai.cypher.client.migration.MigrationSelectionBuilder
import ai.cypher.client.migration.MigrationSettingsUiSelections
import ai.cypher.client.migration.MigrationUiPhase
import ai.cypher.client.migration.MigrationUiSelections
import ai.cypher.client.migration.MigrationUiState
import ai.cypher.client.migration.groupStatus
import ai.cypher.client.plugin.CypherBundle
import ai.cypher.client.session.views.base.BaseQuestionView
import ai.cypher.client.ui.UiStyle
import ai.cypher.client.ui.layout.Stack
import ai.cypher.rpc.dto.LegacyMigrationDetectionDto
import ai.cypher.rpc.dto.MigrationItemCategoryDto
import com.intellij.ui.components.JBCheckBox
import com.intellij.ui.components.JBLabel
import com.intellij.util.concurrency.annotations.RequiresEdt
import java.awt.BorderLayout
import java.awt.Component
import javax.swing.JComponent
import javax.swing.JPanel

private const val ACTION_SKIP = "skip"
private const val ACTION_MIGRATE = "migrate"
private const val ACTION_DONE = "done"
private const val ACTION_CONTINUE = "continue"

/**
 * Migration selection wizard.
 *
 * Build once; call [update] for every state change. Does not rebuild the component tree.
 */
class MigrationWizardPanel : JPanel(BorderLayout()) {

    // ------ Callbacks ------
    var onSkip: (() -> Unit)? = null
    var onStart: ((MigrationUiSelections) -> Unit)? = null
    var onDone: (() -> Unit)? = null
    var onContinueFromError: (() -> Unit)? = null

    // ------ Migrate screen row state ------
    private val rows = mutableMapOf<MigrationItemCategoryDto, MigrationItemRow>()
    private val settingsRow = MigrationItemRow(CypherBundle.message("migration.row.settings"), MigrationItemCategoryDto.settings)
    private val providerRow = MigrationItemRow(CypherBundle.message("migration.row.providers"), MigrationItemCategoryDto.provider)
    private val mcpRow = MigrationItemRow(CypherBundle.message("migration.row.mcp"), MigrationItemCategoryDto.mcpServer)
    private val modesRow = MigrationItemRow(CypherBundle.message("migration.row.modes"), MigrationItemCategoryDto.customMode)
    private val sessionsRow = MigrationItemRow(CypherBundle.message("migration.row.sessions"), MigrationItemCategoryDto.session)
    private val modelRow = MigrationItemRow(CypherBundle.message("migration.row.model"), MigrationItemCategoryDto.defaultModel)

    private val question = BaseQuestionView()
    private val keepBox = JBCheckBox(CypherBundle.message("migration.keep_legacy_settings"), true)

    private val emptyLabel = JBLabel(CypherBundle.message("migration.empty")).apply {
        foreground = UiStyle.Colors.weak()
    }

    private var detection: LegacyMigrationDetectionDto? = null
    private var selections = MigrationUiSelections()
    private var phase = MigrationUiPhase.selecting
    private var running = false

    init {
        isOpaque = false

        rows[MigrationItemCategoryDto.provider] = providerRow
        rows[MigrationItemCategoryDto.mcpServer] = mcpRow
        rows[MigrationItemCategoryDto.customMode] = modesRow
        rows[MigrationItemCategoryDto.session] = sessionsRow
        rows[MigrationItemCategoryDto.defaultModel] = modelRow
        rows[MigrationItemCategoryDto.settings] = settingsRow

        for (row in rows.values) {
            row.onSelectionChanged = { _ -> updateMigrateButtonEnabled() }
        }

        question.setHeader(
            CypherBundle.message("migration.migrate.title"),
            CypherBundle.message("migration.migrate.subtitle"),
        )
        question.setContent(buildContent())
        question.setActions(
            listOf(
                BaseQuestionView.Action(ACTION_SKIP, CypherBundle.message("migration.button.skip"), primary = false) {
                    onSkip?.invoke()
                },
                BaseQuestionView.Action(ACTION_MIGRATE, CypherBundle.message("migration.button.migrate"), primary = true) {
                    onStart?.invoke(currentSelections())
                },
                BaseQuestionView.Action(ACTION_DONE, CypherBundle.message("migration.button.done"), primary = true) {
                    onDone?.invoke()
                },
                BaseQuestionView.Action(ACTION_CONTINUE, CypherBundle.message("migration.button.continue"), primary = true) {
                    onContinueFromError?.invoke()
                },
            ),
        )
        question.setActionLeft(keepBox)

        add(question, BorderLayout.CENTER)
        updateButtons(MigrationUiPhase.selecting, running = false)
    }

    // ------ Public update ------

    @RequiresEdt
    fun update(state: MigrationUiState.Needed) {
        val det = state.detection
        // Detection and default selections are set once on first update and are stable for the
        // lifetime of a single wizard session. The service never re-detects mid-session.
        if (detection == null || detection != det) {
            detection = det
            selections = MigrationSelectionBuilder.defaults(det)
            applyDefaults(det)
        }

        phase = state.phase
        running = state.running

        // Update row visibility based on what data exists
        providerRow.isVisible = det.providers.any { it.supported }
        mcpRow.isVisible = det.mcpServers.isNotEmpty()
        modesRow.isVisible = det.customModes.isNotEmpty()
        sessionsRow.isVisible = det.sessions.isNotEmpty()
        modelRow.isVisible = det.defaultModel != null
        settingsRow.isVisible = det.settings != null
        emptyLabel.isVisible = !det.hasData

        // Update phase for all rows
        for (row in rows.values) {
            row.updatePhase(phase)
        }

        // Update progress for each row category
        updateRowProgress(MigrationItemCategoryDto.provider, state.progress)
        updateRowProgress(MigrationItemCategoryDto.mcpServer, state.progress)
        updateRowProgress(MigrationItemCategoryDto.customMode, state.progress)
        updateRowProgress(MigrationItemCategoryDto.session, state.progress)
        updateRowProgress(MigrationItemCategoryDto.defaultModel, state.progress)
        updateRowProgress(MigrationItemCategoryDto.settings, state.progress)

        updateButtons(phase, running)
        updateMigrateButtonEnabled()
        question.revalidate()
        question.repaint()
        revalidate()
        repaint()
    }

    @RequiresEdt
    fun preferredFocusComponent(): JComponent = question.preferredActionComponent(ACTION_MIGRATE)

    internal fun keepLegacySettingsFileSelectedForTest() = keepBox.isSelected

    // ------ Internal helpers ------

    private fun applyDefaults(det: LegacyMigrationDetectionDto) {
        val defaults = MigrationSelectionBuilder.defaults(det)
        providerRow.selected = defaults.providers.isNotEmpty()
        mcpRow.selected = defaults.mcpServers.isNotEmpty()
        modesRow.selected = defaults.customModes.isNotEmpty()
        sessionsRow.selected = defaults.sessions.isNotEmpty()
        modelRow.selected = defaults.defaultModel
        settingsRow.selected = defaults.settings.autoApproval.commandRules ||
                defaults.settings.autoApproval.readPermission ||
                defaults.settings.autoApproval.writePermission ||
                defaults.settings.autoApproval.executePermission ||
                defaults.settings.autoApproval.mcpPermission ||
                defaults.settings.autoApproval.taskPermission ||
                defaults.settings.language ||
                defaults.settings.autocomplete
        keepBox.isSelected = defaults.keepLegacySettingsFile
    }

    private fun updateRowProgress(category: MigrationItemCategoryDto, items: List<MigrationItemUiProgress>) {
        val row = rows[category] ?: return
        val categoryItems = items.filter { it.category == category }
        if (categoryItems.isEmpty()) {
            row.updateProgress(null)
            return
        }
        val status = groupStatus(categoryItems)
        row.updateProgress(MigrationItemUiProgress(category.name, category, status))
    }

    private fun updateButtons(phase: MigrationUiPhase, running: Boolean) {
        question.setActionVisible(ACTION_SKIP, phase == MigrationUiPhase.selecting)
        question.setActionVisible(ACTION_MIGRATE, phase == MigrationUiPhase.selecting || phase == MigrationUiPhase.migrating)
        question.setActionText(
            ACTION_MIGRATE,
            if (running) CypherBundle.message("migration.button.migrating") else CypherBundle.message("migration.button.migrate"),
        )
        question.setActionVisible(ACTION_DONE, phase == MigrationUiPhase.done)
        question.setActionVisible(ACTION_CONTINUE, phase == MigrationUiPhase.error)
        keepBox.isVisible = phase == MigrationUiPhase.selecting
    }

    private fun updateMigrateButtonEnabled() {
        val any = rows.values.any { it.isVisible && it.selected }
        question.setActionEnabled(ACTION_MIGRATE, any && phase == MigrationUiPhase.selecting && !running)
    }

    private fun currentSelections(): MigrationUiSelections {
        val det = detection ?: return MigrationUiSelections(keepLegacySettingsFile = keepBox.isSelected)
        val providers = if (providerRow.selected) det.providers.filter { it.supported && it.hasApiKey }.map { it.profileName } else emptyList()
        val mcpServers = if (mcpRow.selected) det.mcpServers.map { it.name } else emptyList()
        val modes = if (modesRow.selected) det.customModes.map { it.slug } else emptyList()
        val sessions = if (sessionsRow.selected) det.sessions.map { it.id } else emptyList()
        val defaults = MigrationSelectionBuilder.defaults(det)
        return MigrationUiSelections(
            providers = providers,
            mcpServers = mcpServers,
            customModes = modes,
            sessions = sessions,
            defaultModel = modelRow.selected,
            settings = if (settingsRow.selected) defaults.settings else MigrationSettingsUiSelections(),
            keepLegacySettingsFile = keepBox.isSelected,
        )
    }

    private fun buildContent(): JComponent {
        return Stack.vertical(gap = UiStyle.Gap.xs()).apply {
            alignmentX = Component.LEFT_ALIGNMENT
        }
            .next(emptyLabel)
            .next(providerRow)
            .next(mcpRow)
            .next(modesRow)
            .next(sessionsRow)
            .next(modelRow)
            .next(settingsRow)
    }
}
