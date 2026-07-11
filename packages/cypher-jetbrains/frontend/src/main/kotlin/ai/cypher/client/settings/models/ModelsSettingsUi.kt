package ai.cypher.client.settings.models

import ai.cypher.client.app.CypherAppService
import ai.cypher.client.app.CypherWorkspaceService
import ai.cypher.client.plugin.CypherBundle
import ai.cypher.client.session.ui.ReasoningPicker
import ai.cypher.client.session.ui.model.ModelPicker
import ai.cypher.client.session.ui.model.ModelText
import ai.cypher.client.settings.base.BaseContentPanel
import ai.cypher.client.settings.base.BaseSettingsUi
import ai.cypher.client.settings.base.SettingsBannerKind
import ai.cypher.client.settings.base.SettingsRow
import ai.cypher.client.settings.base.SettingsRows
import ai.cypher.client.ui.layout.HAlign
import ai.cypher.client.ui.layout.VAlign
import ai.cypher.client.ui.layout.align
import ai.cypher.log.CypherLog
import ai.cypher.rpc.dto.AgentDto
import ai.cypher.rpc.dto.ConfigPatchDto
import ai.cypher.rpc.dto.CypherAppStateDto
import ai.cypher.rpc.dto.CypherAppStatusDto
import ai.cypher.rpc.dto.LoadErrorDto
import ai.cypher.rpc.dto.ModelStateDto
import ai.cypher.rpc.dto.ModelsWorkspaceDto
import ai.cypher.rpc.dto.ProvidersDto
import com.intellij.openapi.components.service
import com.intellij.util.concurrency.annotations.RequiresEdt
import kotlinx.coroutines.CoroutineScope

internal class ModelsSettingsUi(
    cs: CoroutineScope,
    private val app: CypherAppService = service(),
    private val workspaces: CypherWorkspaceService = service(),
    directory: String? = null,
) : BaseSettingsUi<ModelsSettingsContent, ModelsDraft, ConfigPatchDto, CypherAppStateDto, ModelsWorkspaceDto>(
    cs,
    ModelsDraft(),
    app,
    workspaces,
    directory,
) {

    companion object {
        private val LOG = CypherLog.create(ModelsSettingsUi::class.java)
    }

    private val defaults get() = form.defaults
    private val small get() = form.small
    private val subagent get() = form.subagent
    private val variant get() = form.variant
    private val variantRow get() = form.variantRow
    private val pickers get() = form.pickers

    private var providers: ProvidersDto? = null
    private var agents: List<AgentDto> = emptyList()
    private var errors: List<LoadErrorDto> = emptyList()
    private var allItems: List<ModelPicker.Item> = emptyList()

    init {
        startSettings(ModelsSettingsContent(app, { updateDraft(it) }, ::selectSubagent))
    }

    override fun change(from: ModelsDraft, to: ModelsDraft): ConfigPatchDto? = patch(from, to).takeIf {
        it.values.isNotEmpty() || it.agents.isNotEmpty()
    }

    override fun save(change: ConfigPatchDto, done: (CypherAppStateDto?) -> Unit) {
        app.updateConfigAsync(change, done)
    }

    override fun base(result: CypherAppStateDto): ModelsDraft = modelsDraft(result.config, agents)

    override fun draft(state: CypherAppStateDto): ModelsDraft = modelsDraft(state.config, agents)

    override fun saved(base: ModelsDraft, draft: ModelsDraft): Boolean = savedMatches(base, draft)

    override fun pendingText(): String = CypherBundle.message("settings.models.save.pending")

    override fun failedText(): String = CypherBundle.message("settings.models.save.failed")

    override fun logSaveStarted(change: ConfigPatchDto) = LOG.info("model settings save: started ${summary(change)}")

    override fun logSaveCompleted(change: ConfigPatchDto) = LOG.info("model settings save: completed ${summary(change)}")

    override fun logSaveFailed(change: ConfigPatchDto) = LOG.warn("model settings save: failed ${summary(change)}")

    override fun logSaveFailedAfterDispose(change: ConfigPatchDto) = LOG.warn("model settings save: failed after dispose ${summary(change)}")

    override fun logSaveCompletedAfterDispose(change: ConfigPatchDto) = LOG.info("model settings save: completed after dispose ${summary(change)}")

    override fun unavailable(state: CypherAppStateDto) {
        if (!workspaceLoaded && providers == null) {
            agents = emptyList()
            errors = emptyList()
        }
    }

    override fun models(state: ModelStateDto) = Unit

    override fun clearWorkspaceError() {
        errors = emptyList()
    }

    override suspend fun loadWorkspace(root: String): ModelsWorkspaceDto = workspaces.models(root)

    override fun applyWorkspace(result: ModelsWorkspaceDto) {
        providers = result.providers
        agents = result.agents?.agents ?: emptyList()
        errors = result.errors
    }

    @RequiresEdt
    override fun syncContent() {
        allItems = items(false)
        val smallItems = items(true)
        val state = modelsStatus(
            ready = appState.status == CypherAppStatusDto.READY && hasProjectDirectory,
            loading = workspaceLoading || (appState.status == CypherAppStatusDto.READY && !workspaceLoaded && hasProjectDirectory),
            providers = providers,
            items = allItems.size,
            errors = errors,
            saving = saving,
        )
        val ready = state == ModelsStatus.READY || state == ModelsStatus.MODES_FAILED
        val editable = !saving && (ready || state == ModelsStatus.LOADING)
        val bannerVisible = modelsLoginBannerVisible(
            ready = appState.status == CypherAppStatusDto.READY,
            authenticated = appState.profile != null,
        )
        syncModelBanner(state, bannerVisible)
        val err = saveError
        if (saving || state == ModelsStatus.SAVING) {
            showProgress(CypherBundle.message("settings.models.save.pending"))
        } else if (err != null) {
            showError(err)
        } else if (state == ModelsStatus.UNAVAILABLE || state == ModelsStatus.LOADING) {
            showProgress(CypherBundle.message("settings.models.loading"))
        } else {
            clearProgress()
        }
        var layout = false
        defaults.setItems(allItems, draft.model)
        small.setItems(smallItems, draft.small)
        subagent.setItems(allItems, draft.subagent)
        listOf(defaults, small, subagent).forEach { it.isEnabled = editable }
        layout = syncVariant(editable) || layout
        layout = syncModes(editable) || layout
        if (layout) {
            revalidate()
            repaint()
        }
    }

    @RequiresEdt
    private fun syncModelBanner(state: ModelsStatus, login: Boolean) {
        syncLoginBanner(login) {
            if ((saving || state == ModelsStatus.LOADING || state == ModelsStatus.SAVING) && top.isVisible) return@syncLoginBanner
            when (state) {
                ModelsStatus.LOAD_FAILED -> top.showBanner(
                    CypherBundle.message("settings.models.load.failed"),
                    emptyList(),
                    SettingsBannerKind.ERROR,
                )
                ModelsStatus.NO_PROVIDERS -> top.showBanner(CypherBundle.message("settings.models.noProviders"), emptyList())
                ModelsStatus.MODES_FAILED -> top.showBanner(CypherBundle.message("settings.models.modes.failed"), emptyList())
                else -> top.hideBanner()
            }
        }
    }

    private fun items(includeSmall: Boolean): List<ModelPicker.Item> {
        val cfg = providers ?: return emptyList()
        return cfg.providers
            .filter { it.id == CYPHER_PROVIDER || it.id in cfg.connected }
            .flatMap { provider ->
                provider.models.mapNotNull { (id, model) ->
                    val item = ModelPicker.Item(
                        id = id,
                        display = model.name,
                        provider = provider.id,
                        providerName = provider.name,
                        inputPrice = model.inputPrice,
                        outputPrice = model.outputPrice,
                        contextLength = model.contextLength,
                        releaseDate = model.releaseDate,
                        latest = model.latest,
                        recommendedIndex = model.recommendedIndex,
                        free = model.free,
                        byok = model.byok,
                        variants = model.variants,
                        limit = model.limit,
                        cost = model.cost,
                        capabilities = model.capabilities,
                        options = model.options,
                        autoRouting = model.autoRouting,
                        terminalBench = model.terminalBench,
                        reasoning = model.reasoning,
                        attachment = model.attachment,
                        mayTrainOnYourPrompts = model.mayTrainOnYourPrompts,
                    )
                    if (!includeSmall && ModelText.small(item)) return@mapNotNull null
                    item
                }
            }
    }

    @RequiresEdt
    private fun syncVariant(ready: Boolean): Boolean {
        val item = allItems.firstOrNull { it.key == draft.subagent || it.id == draft.subagent }
        val valid = item?.variants.orEmpty()
        if (draft.variant != null && draft.variant !in valid) draft = draft.copy(variant = valid.firstOrNull())
        if (draft.subagent != null && valid.isEmpty() && draft.variant != null) draft = draft.copy(variant = null)
        variant.setItems(valid.map { ReasoningPicker.Item(it, variantTitle(it)) }, draft.variant)
        variant.isEnabled = ready && valid.isNotEmpty()
        val visible = valid.isNotEmpty()
        val changed = variantRow.isVisible != visible
        variantRow.isVisible = visible
        variant.isVisible = visible
        return changed
    }

    @RequiresEdt
    private fun syncModes(ready: Boolean): Boolean {
        var layout = false
        val names = agents.map { it.name }
        if (names != pickers.keys.toList()) {
            form.modes.removeAll()
            pickers.clear()
            agents.forEach { agent ->
                val picker = ModelSettingPicker()
                picker.picker.favorites = { app.favorites.value }
                picker.picker.onFavoriteToggle = { app.toggleModelFavorite(it.provider, it.id) }
                picker.picker.onSelect = { item -> updateDraft { copy(agents = this.agents + (agent.name to item.key)) } }
                picker.picker.onClear = { updateDraft { copy(agents = this.agents + (agent.name to null)) } }
                pickers[agent.name] = picker
                form.modes.row(agent.name, SettingsRow(
                    agent.displayName ?: title(agent.name),
                    agent.description,
                    picker,
                ))
            }
            layout = true
        }
        agents.forEach { agent ->
            val name = agent.name
            val picker = pickers[name] ?: return@forEach
            form.modes.update(name, agent.displayName ?: title(name), agent.description, picker)
            val value = draft.agents[name]
            picker.setItems(allItems, value)
            picker.isEnabled = ready
        }
        return layout
    }

    private fun selectSubagent(item: ModelPicker.Item) {
        val variant = if (draft.subagent == item.key && draft.variant in item.variants) draft.variant else item.variants.firstOrNull()
        updateDraft { copy(subagent = item.key, variant = variant) }
    }
}

private const val CYPHER_PROVIDER = "cypher"

private fun summary(patch: ConfigPatchDto): String {
    val values = patch.values.keys.sorted().joinToString(",").ifEmpty { "none" }
    return "values=$values agents=${patch.agents.size}"
}

internal class ModelsSettingsContent(
    app: CypherAppService,
    update: (ModelsDraft.() -> ModelsDraft) -> Unit,
    select: (ModelPicker.Item) -> Unit,
) : BaseContentPanel() {
    val defaults = ModelSettingPicker()
    val small = ModelSettingPicker()
    val subagent = ModelSettingPicker()
    val variant = ReasoningPicker()
    val variantRow = SettingsRow(
        CypherBundle.message("settings.models.subagentVariant.title"),
        CypherBundle.message("settings.models.subagentVariant.description"),
        variant.align(HAlign.RIGHT, VAlign.CENTER),
    )
    val modes: SettingsRows
    val pickers = linkedMapOf<String, ModelSettingPicker>()

    init {
        defaults.picker.onSelect = { update { copy(model = it.key) } }
        defaults.picker.onClear = { update { copy(model = null) } }
        small.picker.onSelect = { update { copy(small = it.key) } }
        small.picker.onClear = { update { copy(small = null) } }
        small.picker.includeSmall = true
        subagent.picker.onSelect = { item -> select(item) }
        subagent.picker.onClear = { update { copy(subagent = null, variant = null) } }
        variant.onSelect = { item -> update { copy(variant = item.id) } }
        listOf(defaults, small, subagent).forEach { picker ->
            picker.picker.favorites = { app.favorites.value }
            picker.picker.onFavoriteToggle = { app.toggleModelFavorite(it.provider, it.id) }
        }

        val rows = SettingsRows()
        next(rows)
        rows.row(SettingsRow(
            CypherBundle.message("settings.models.defaultModel.title"),
            CypherBundle.message("settings.models.defaultModel.description"),
            defaults,
        ))
        rows.row(SettingsRow(
            CypherBundle.message("settings.models.smallModel.title"),
            CypherBundle.message("settings.models.smallModel.description"),
            small,
        ))
        rows.row(SettingsRow(
            CypherBundle.message("settings.models.subagentModel.title"),
            CypherBundle.message("settings.models.subagentModel.description"),
            subagent,
        ))
        rows.row(variantRow)
        modes = section(
            CypherBundle.message("settings.models.modeModels.title"),
            CypherBundle.message("settings.models.modeModels.description"),
        )
    }
}

private fun variantTitle(value: String): String = value.replaceFirstChar { it.titlecase() }

private fun title(value: String): String = value.replace('-', ' ').replace('_', ' ').replaceFirstChar { it.titlecase() }
