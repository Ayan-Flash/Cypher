@file:Suppress("UnstableApiUsage")

package ai.cypher.rpc

import ai.cypher.rpc.dto.LegacyCleanupReportDto
import ai.cypher.rpc.dto.LegacyCleanupTargetsDto
import ai.cypher.rpc.dto.LegacyMigrationDetectionDto
import ai.cypher.rpc.dto.LegacyMigrationEventDto
import ai.cypher.rpc.dto.LegacyMigrationSelectionsDto
import ai.cypher.rpc.dto.LegacyMigrationStatusDto
import com.intellij.platform.rpc.RemoteApiProviderService
import fleet.rpc.RemoteApi
import fleet.rpc.Rpc
import fleet.rpc.remoteApiDescriptor
import kotlinx.coroutines.flow.Flow

/**
 * App-level RPC API for legacy migration operations.
 *
 * All operations are app-scoped. The backend implementation delegates to
 * [ai.cypher.backend.app.CypherBackendMigrationManager] using the active CLI connection.
 */
@Rpc
interface CypherMigrationRpcApi : RemoteApi<Unit> {
    companion object {
        suspend fun getInstance(): CypherMigrationRpcApi =
            RemoteApiProviderService.resolve(remoteApiDescriptor<CypherMigrationRpcApi>())
    }

    /** Return the persisted migration status, or null if not yet set. */
    suspend fun status(): LegacyMigrationStatusDto?

    /** Detect legacy data and return a summary of what can be migrated. */
    suspend fun detect(): LegacyMigrationDetectionDto

    /** Run migration for the given selections, streaming progress events. */
    suspend fun migrate(selections: LegacyMigrationSelectionsDto): Flow<LegacyMigrationEventDto>

    /** Mark migration as skipped. */
    suspend fun skip()

    /** Mark migration as completed or completed with errors. */
    suspend fun finalize(status: LegacyMigrationStatusDto)

    /** Clean up legacy data after migration. */
    suspend fun cleanup(targets: LegacyCleanupTargetsDto): LegacyCleanupReportDto
}
