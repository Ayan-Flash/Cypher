@file:Suppress("UnstableApiUsage")

package ai.cypher.backend.rpc

import ai.cypher.backend.app.CypherAppState
import ai.cypher.backend.app.CypherBackendAppService
import ai.cypher.backend.telemetry.CypherBackendTelemetry
import ai.cypher.backend.app.ConfigWarning
import ai.cypher.backend.app.LoadError
import ai.cypher.backend.app.LoadProgress
import ai.cypher.backend.app.ProfileResult
import ai.cypher.backend.cli.CypherCliPlatform
import ai.cypher.backend.cli.CypherProps
import ai.cypher.jetbrains.api.model.CypherProfile200Response
import ai.cypher.rpc.dto.ConfigPatchDto
import ai.cypher.rpc.CypherAppRpcApi
import ai.cypher.rpc.dto.ConfigWarningDto
import ai.cypher.rpc.dto.DeviceAuthDto
import ai.cypher.rpc.dto.HealthDto
import ai.cypher.rpc.dto.CypherAppStateDto
import ai.cypher.rpc.dto.CypherAppStatusDto
import ai.cypher.rpc.dto.LoadErrorDto
import ai.cypher.rpc.dto.LoadProgressDto
import ai.cypher.rpc.dto.ModelFavoriteUpdateDto
import ai.cypher.rpc.dto.ModelSelectionUpdateDto
import ai.cypher.rpc.dto.ModelStateDto
import ai.cypher.rpc.dto.ModelVariantUpdateDto
import ai.cypher.rpc.dto.ProfileBalanceDto
import ai.cypher.rpc.dto.ProfileDto
import ai.cypher.rpc.dto.ProfileCypherPassDto
import ai.cypher.rpc.dto.ProfileOrganizationDto
import ai.cypher.rpc.dto.ProfileStatusDto
import ai.cypher.rpc.dto.TelemetryCaptureDto
import com.intellij.openapi.components.service
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map

/**
 * Backend implementation of [CypherAppRpcApi].
 *
 * Delegates directly to the app-level [CypherBackendAppService] —
 * no project resolution needed since all operations are app-scoped.
 */
class CypherAppRpcApiImpl : CypherAppRpcApi {

    private val app: CypherBackendAppService get() = service()

    override suspend fun connect() = app.connect()

    override suspend fun state(): Flow<CypherAppStateDto> =
        app.appState.map(::dto).distinctUntilChanged()

    override suspend fun health(): HealthDto = app.health()

    override suspend fun cliVersion(): String = CypherProps.cliVersion()

    override suspend fun cliPlatform(): String = CypherCliPlatform.current()

    override suspend fun retry() = app.retry()

    override suspend fun restart() = app.restart()

    override suspend fun reinstall() = app.reinstall()

    override suspend fun modelState(): ModelStateDto {
        app.requireReady()
        return app.models.state()
    }

    override suspend fun updateModelFavorite(update: ModelFavoriteUpdateDto): ModelStateDto {
        app.requireReady()
        return app.models.favorite(update)
    }

    override suspend fun updateModelSelection(update: ModelSelectionUpdateDto): ModelStateDto {
        app.requireReady()
        return app.models.selection(update)
    }

    override suspend fun clearModelSelection(agent: String): ModelStateDto {
        app.requireReady()
        return app.models.clear(agent)
    }

    override suspend fun updateModelVariant(update: ModelVariantUpdateDto): ModelStateDto {
        app.requireReady()
        return app.models.variant(update)
    }

    override suspend fun updateConfig(patch: ConfigPatchDto): CypherAppStateDto {
        app.requireReady()
        return appStateDto(app.updateConfig(patch))
    }

    override suspend fun refreshProfile(): ProfileDto? = app.refreshProfile()?.let(::profileDto)

    override suspend fun startLogin(directory: String?): DeviceAuthDto = app.startLogin(directory)

    override suspend fun completeLogin(directory: String?): ProfileDto? = app.completeLogin(directory)?.let(::profileDto)

    override suspend fun logout(): Boolean = app.logout()

    override suspend fun setOrganization(organizationId: String?): ProfileDto? =
        app.setOrganization(organizationId)?.let(::profileDto)

    override suspend fun captureTelemetry(capture: TelemetryCaptureDto) {
        service<CypherBackendTelemetry>().capture(app.http, app.port, capture.event, capture.properties)
    }

    private fun dto(state: CypherAppState): CypherAppStateDto =
        appStateDto(state)
}

internal fun appStateDto(state: CypherAppState): CypherAppStateDto =
    when (state) {
        CypherAppState.Disconnected -> CypherAppStateDto(CypherAppStatusDto.DISCONNECTED)
        is CypherAppState.Downloading -> CypherAppStateDto(
            status = CypherAppStatusDto.DOWNLOADING,
            downloadPercent = state.percent,
            downloadVersion = state.version,
            downloadPlatform = state.platform,
        )
        CypherAppState.Connecting -> CypherAppStateDto(CypherAppStatusDto.CONNECTING)
        is CypherAppState.Loading -> CypherAppStateDto(
            status = CypherAppStatusDto.LOADING,
            progress = progress(state.progress),
        )
        is CypherAppState.MigrationRequired -> CypherAppStateDto(
            status = CypherAppStatusDto.MIGRATION_REQUIRED,
            migration = MigrationRpcMapper.toDto(state.detection),
        )
        is CypherAppState.Ready -> CypherAppStateDto(
            status = CypherAppStatusDto.READY,
            progress = LoadProgressDto(
                config = true,
                notifications = true,
                profile = if (state.data.profile != null) ProfileStatusDto.LOADED
                    else ProfileStatusDto.NOT_LOGGED_IN,
            ),
            warnings = state.data.warnings.map(::warning),
            config = state.data.config,
            profile = state.data.profile?.let(::profileDto),
        )
        is CypherAppState.Error -> CypherAppStateDto(
            status = CypherAppStatusDto.ERROR,
            error = state.message,
            errors = state.errors.map(::error),
        )
    }

internal fun profileDto(p: CypherProfile200Response): ProfileDto = ProfileDto(
    email = p.profile.email,
    name = p.profile.name,
    organizations = p.profile.organizations.orEmpty().map { org ->
        ProfileOrganizationDto(id = org.id, name = org.name, role = org.role)
    },
    // The pinned CLI release does not expose hasPersonalAccount yet, so default to
    // showing the personal account. Flip back to p.profile.hasPersonalAccount once a
    // CLI release ships the field.
    hasPersonalAccount = true,
    balance = p.balance?.balance?.let { ProfileBalanceDto(balance = it) },
    cypherPass = p.cypherPass?.let {
        val base = it.currentPeriodBaseCreditsUsd ?: return@let null
        val usage = it.currentPeriodUsageUsd ?: return@let null
        val bonus = it.currentPeriodBonusCreditsUsd ?: return@let null
        ProfileCypherPassDto(
            currentPeriodBaseCreditsUsd = base,
            currentPeriodUsageUsd = usage,
            currentPeriodBonusCreditsUsd = bonus,
            nextBillingAt = it.nextBillingAt,
        )
    },
    currentOrgId = p.currentOrgId,
)

private fun progress(p: LoadProgress) = LoadProgressDto(
    config = p.config,
    notifications = p.notifications,
    profile = when (p.profile) {
        ProfileResult.PENDING -> ProfileStatusDto.PENDING
        ProfileResult.LOADED -> ProfileStatusDto.LOADED
        ProfileResult.NOT_LOGGED_IN -> ProfileStatusDto.NOT_LOGGED_IN
    },
)

private fun error(e: LoadError) = LoadErrorDto(
    resource = e.resource,
    status = e.status,
    detail = e.detail,
)

private fun warning(w: ConfigWarning) = ConfigWarningDto(
    path = w.path,
    message = w.message,
    detail = w.detail,
)
