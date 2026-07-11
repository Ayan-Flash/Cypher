package ai.cypher.backend.app

import ai.cypher.jetbrains.api.model.CypherNotifications200ResponseInner
import ai.cypher.jetbrains.api.model.CypherProfile200Response
import ai.cypher.backend.migration.LegacyMigrationDetection
import ai.cypher.rpc.dto.ConfigDto

/**
 * Full application lifecycle state, combining CLI transport connection
 * status with data-loading progress.
 *
 * [ConnectionState] stays internal to [CypherConnectionService] for the
 * transport layer. This sealed class is what the frontend observes.
 */
sealed class CypherAppState {
    data object Disconnected : CypherAppState()
    data class Downloading(val percent: Int, val version: String, val platform: String) : CypherAppState()
    data object Connecting : CypherAppState()
    data class Loading(val progress: LoadProgress) : CypherAppState()
    data class MigrationRequired(val detection: LegacyMigrationDetection) : CypherAppState()
    data class Ready(val data: AppData, val rev: Long = 0) : CypherAppState()
    data class Error(val message: String, val errors: List<LoadError> = emptyList()) : CypherAppState()
}

/**
 * Tracks which global data fetches have completed during the [CypherAppState.Loading] phase.
 */
data class LoadProgress(
    val config: Boolean = false,
    val notifications: Boolean = false,
    val profile: ProfileResult = ProfileResult.PENDING,
)

/** Outcome of the profile fetch. */
enum class ProfileResult { PENDING, LOADED, NOT_LOGGED_IN }

/**
 * Error detail for a single resource that failed to load.
 */
data class LoadError(
    val resource: String,
    val status: Int? = null,
    val detail: String? = null,
)

data class ConfigWarning(
    val path: String,
    val message: String,
    val detail: String? = null,
)

/**
 * All global data that has been successfully loaded.
 * Present only in [CypherAppState.Ready].
 */
data class AppData(
    val profile: CypherProfile200Response?,
    val config: ConfigDto,
    val notifications: List<CypherNotifications200ResponseInner>,
    val warnings: List<ConfigWarning> = emptyList(),
)
