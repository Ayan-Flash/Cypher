package ai.cypher.client.app

import ai.cypher.rpc.dto.CypherWorkspaceStateDto
import kotlinx.coroutines.flow.StateFlow

/**
 * A workspace for a single directory. Mirrors the CLI concept of a
 * workspace — a directory with its providers, agents, commands, skills.
 *
 * Immutable reference — [state] flows internally as the workspace loads.
 * Lifecycle managed by [CypherWorkspaceService].
 */
class Workspace(
    val directory: String,
    val state: StateFlow<CypherWorkspaceStateDto>,
    val reload: () -> Unit,
)
