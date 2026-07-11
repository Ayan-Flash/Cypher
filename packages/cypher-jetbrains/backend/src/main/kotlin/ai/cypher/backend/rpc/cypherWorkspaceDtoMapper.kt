package ai.cypher.backend.rpc

import ai.cypher.backend.app.LoadError
import ai.cypher.backend.workspace.AgentData
import ai.cypher.backend.workspace.AgentInfo
import ai.cypher.backend.workspace.CommandInfo
import ai.cypher.backend.workspace.CypherWorkspaceLoadProgress
import ai.cypher.backend.workspace.ModelInfo
import ai.cypher.backend.workspace.ProviderData
import ai.cypher.backend.workspace.ProviderInfo
import ai.cypher.backend.workspace.SkillInfo
import ai.cypher.rpc.dto.ModelAutoRoutingDto
import ai.cypher.rpc.dto.ModelCacheCostDto
import ai.cypher.rpc.dto.ModelCapabilitiesDto
import ai.cypher.rpc.dto.ModelCostDto
import ai.cypher.rpc.dto.AgentDto
import ai.cypher.rpc.dto.AgentsDto
import ai.cypher.rpc.dto.CommandDto
import ai.cypher.rpc.dto.CypherWorkspaceLoadProgressDto
import ai.cypher.rpc.dto.LoadErrorDto
import ai.cypher.rpc.dto.ModelDto
import ai.cypher.rpc.dto.ModelInputCapabilitiesDto
import ai.cypher.rpc.dto.ModelLimitDto
import ai.cypher.rpc.dto.ModelOptionsDto
import ai.cypher.rpc.dto.ModelTerminalBenchDto
import ai.cypher.rpc.dto.ProviderDto
import ai.cypher.rpc.dto.ProvidersDto
import ai.cypher.rpc.dto.SkillDto

internal object CypherWorkspaceDtoMapper {
    fun error(e: LoadError) = LoadErrorDto(
        resource = e.resource,
        status = e.status,
        detail = e.detail,
    )

    fun progress(p: CypherWorkspaceLoadProgress) = CypherWorkspaceLoadProgressDto(
        providers = p.providers,
        agents = p.agents,
        commands = p.commands,
        skills = p.skills,
    )

    fun providers(d: ProviderData) = ProvidersDto(
        providers = d.providers.map(::provider),
        connected = d.connected,
        defaults = d.defaults,
    )

    fun agents(d: AgentData) = AgentsDto(
        agents = d.agents.map(::agent),
        all = d.all.map(::agent),
        default = d.default,
    )

    fun command(c: CommandInfo) = CommandDto(
        name = c.name,
        description = c.description,
        source = c.source,
        hints = c.hints,
    )

    fun skill(s: SkillInfo) = SkillDto(
        name = s.name,
        description = s.description,
        location = s.location,
    )

    private fun provider(p: ProviderInfo) = ProviderDto(
        id = p.id,
        name = p.name,
        source = p.source,
        models = p.models.mapValues { (_, m) -> model(m) },
    )

    private fun model(m: ModelInfo) = ModelDto(
        id = m.id,
        name = m.name,
        inputPrice = m.inputPrice,
        outputPrice = m.outputPrice,
        contextLength = m.contextLength,
        releaseDate = m.releaseDate,
        latest = m.latest,
        attachment = m.attachment,
        reasoning = m.reasoning,
        temperature = m.temperature,
        toolCall = m.toolCall,
        free = m.free,
        byok = m.byok,
        status = m.status,
        recommendedIndex = m.recommendedIndex,
        variants = m.variants,
        limit = m.limit?.let { ModelLimitDto(it.context, it.input, it.output) },
        cost = m.cost?.let { cost ->
            ModelCostDto(
                input = cost.input,
                output = cost.output,
                cache = cost.cache?.let { ModelCacheCostDto(it.read, it.write) },
            )
        },
        capabilities = m.capabilities?.let { cap ->
            ModelCapabilitiesDto(
                reasoning = cap.reasoning,
                input = cap.input?.let { ModelInputCapabilitiesDto(it.text, it.image, it.audio, it.video, it.pdf) },
            )
        },
        options = m.options?.let { ModelOptionsDto(it.description) },
        autoRouting = m.autoRouting?.let { ModelAutoRoutingDto(it.models) },
        terminalBench = m.terminalBench?.let { ModelTerminalBenchDto(it.overallScore, it.avgAttemptCostUsd) },
        mayTrainOnYourPrompts = m.mayTrainOnYourPrompts,
    )

    private fun agent(a: AgentInfo) = AgentDto(
        name = a.name,
        displayName = a.displayName,
        description = a.description,
        mode = a.mode,
        native = a.native,
        hidden = a.hidden,
        color = a.color,
        deprecated = a.deprecated,
    )
}
