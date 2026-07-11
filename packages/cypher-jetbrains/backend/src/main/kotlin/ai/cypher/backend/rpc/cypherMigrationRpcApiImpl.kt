@file:Suppress("UnstableApiUsage")

package ai.cypher.backend.rpc

import ai.cypher.backend.app.CypherBackendAppService
import ai.cypher.backend.migration.CypherBackendLegacyMigrationStoreService
import ai.cypher.backend.migration.LegacyMigrationResultItem
import ai.cypher.backend.migration.LegacyMigrationSink
import ai.cypher.backend.migration.LegacyMigrationStatus
import ai.cypher.backend.migration.MigrationItemCategory
import ai.cypher.backend.migration.MigrationItemStatus
import ai.cypher.rpc.CypherMigrationRpcApi
import ai.cypher.rpc.dto.LegacyCleanupReportDto
import ai.cypher.rpc.dto.LegacyCleanupTargetsDto
import ai.cypher.rpc.dto.LegacyMigrationDetectionDto
import ai.cypher.rpc.dto.LegacyMigrationEventDto
import ai.cypher.rpc.dto.LegacyMigrationSelectionsDto
import ai.cypher.rpc.dto.LegacyMigrationStatusDto
import ai.cypher.backend.app.CypherBackendMigrationManager
import ai.cypher.log.CypherLog
import com.intellij.openapi.components.service
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.trySendBlocking
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.channelFlow
import kotlinx.coroutines.withContext

class CypherMigrationRpcApiImpl : CypherMigrationRpcApi {

    companion object {
        private val LOG = CypherLog.create(CypherMigrationRpcApiImpl::class.java)
    }

    private val app: CypherBackendAppService get() = service()
    private val storeService: CypherBackendLegacyMigrationStoreService get() = service()

    private fun manager(): CypherBackendMigrationManager {
        val http = app.http ?: throw IllegalStateException("Not connected")
        val port = app.port
        return CypherBackendMigrationManager(http, port)
    }

    override suspend fun status(): LegacyMigrationStatusDto? {
        val store = storeService.store()
        val status = withContext(Dispatchers.IO) { store.status() } ?: return null
        LOG.info("Migration RPC status: status=$status")
        return MigrationRpcMapper.toDto(status)
    }

    override suspend fun detect(): LegacyMigrationDetectionDto {
        LOG.info("Migration RPC detect: started")
        val mgr = manager()
        val store = storeService.store()
        val detection = withContext(Dispatchers.IO) { mgr.detect(store) }
        LOG.info("Migration RPC detect: completed hasData=${detection.hasData} providers=${detection.providers.size} mcp=${detection.mcpServers.size} modes=${detection.customModes.size} sessions=${detection.sessions.size}")
        return MigrationRpcMapper.toDto(detection)
    }

    override suspend fun migrate(selections: LegacyMigrationSelectionsDto): Flow<LegacyMigrationEventDto> {
        LOG.info("Migration RPC migrate: starting ${selectionSummary(selections)}")
        val mgr = manager()
        val domainSelections = MigrationRpcMapper.fromDto(selections)
        val store = storeService.store()
        return channelFlow {
            withContext(Dispatchers.IO) {
                val sink = object : LegacyMigrationSink {
                    override fun item(progress: ai.cypher.backend.migration.LegacyMigrationItemProgress) {
                        LOG.info("Migration RPC item: item=${progress.item} status=${progress.status} message=${progress.message}")
                        trySendBlocking(LegacyMigrationEventDto.Item(MigrationRpcMapper.toDto(progress)))
                    }
                    override fun session(progress: ai.cypher.backend.migration.LegacyMigrationSessionProgress) {
                        LOG.info("Migration RPC session: phase=${progress.phase} session=${progress.session?.id} error=${progress.error}")
                        trySendBlocking(LegacyMigrationEventDto.Session(MigrationRpcMapper.toDto(progress)))
                    }
                }
                val report = runCatching {
                    mgr.migrate(store, domainSelections, sink)
                }.getOrElse { e ->
                    val msg = e.message ?: "Migration failed"
                    LOG.warn("Migration RPC migrate: failed message=$msg", e)
                    val errItem = LegacyMigrationResultItem(
                        item = "Migration",
                        category = MigrationItemCategory.settings,
                        status = MigrationItemStatus.error,
                        message = msg,
                    )
                    trySendBlocking(LegacyMigrationEventDto.Complete(listOf(MigrationRpcMapper.toDto(errItem))))
                    return@withContext
                }
                LOG.info("Migration RPC migrate: complete items=${report.items.size} errors=${report.items.count { it.status == MigrationItemStatus.error }}")
                trySendBlocking(LegacyMigrationEventDto.Complete(report.items.map(MigrationRpcMapper::toDto)))
            }
        }
    }

    override suspend fun skip() {
        LOG.info("Migration RPC skip: marking skipped")
        val store = storeService.store()
        withContext(Dispatchers.IO) { store.mark(LegacyMigrationStatus.Skipped) }
        app.resumeAfterMigration()
        LOG.info("Migration RPC skip: resumed app load")
    }

    override suspend fun finalize(status: LegacyMigrationStatusDto) {
        LOG.info("Migration RPC finalize: status=$status")
        val store = storeService.store()
        val domain = MigrationRpcMapper.fromDto(status)
        if (domain != LegacyMigrationStatus.Skipped) {
            withContext(Dispatchers.IO) { store.mark(domain) }
        }
        app.resumeAfterMigration()
        LOG.info("Migration RPC finalize: resumed app load")
    }

    override suspend fun cleanup(targets: LegacyCleanupTargetsDto): LegacyCleanupReportDto {
        LOG.info("Migration RPC cleanup: providerProfiles=${targets.providerProfiles} mcp=${targets.mcpSettings} modes=${targets.customModes} state=${targets.globalState} history=${targets.taskHistory} file=${targets.legacySettingsFile}")
        val mgr = manager()
        val store = storeService.store()
        val report = withContext(Dispatchers.IO) { mgr.cleanup(store, MigrationRpcMapper.fromDto(targets)) }
        LOG.info("Migration RPC cleanup: cleaned=${report.cleaned.size} errors=${report.errors.size}")
        return MigrationRpcMapper.toDto(report)
    }

    private fun selectionSummary(selections: LegacyMigrationSelectionsDto): String =
        "providers=${selections.providers.size} mcp=${selections.mcpServers.size} modes=${selections.customModes.size} sessions=${selections.sessions.size} model=${selections.defaultModel} settings=true keepFile=${selections.keepLegacySettingsFile}"
}
