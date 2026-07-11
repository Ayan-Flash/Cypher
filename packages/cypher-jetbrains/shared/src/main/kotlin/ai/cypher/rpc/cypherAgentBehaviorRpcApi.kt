package ai.cypher.rpc

import ai.cypher.rpc.dto.AgentDetailDto
import ai.cypher.rpc.dto.AgentCreateDto
import ai.cypher.rpc.dto.CommandDto
import ai.cypher.rpc.dto.McpConfigDto
import ai.cypher.rpc.dto.McpServerConfigDto
import ai.cypher.rpc.dto.McpStatusDto
import ai.cypher.rpc.dto.SkillDto
import com.intellij.platform.rpc.RemoteApiProviderService
import fleet.rpc.RemoteApi
import fleet.rpc.Rpc
import fleet.rpc.remoteApiDescriptor

@Rpc
interface CypherAgentBehaviorRpcApi : RemoteApi<Unit> {
    companion object {
        suspend fun getInstance(): CypherAgentBehaviorRpcApi {
            return RemoteApiProviderService.resolve(remoteApiDescriptor<CypherAgentBehaviorRpcApi>())
        }
    }

    suspend fun agents(directory: String): List<AgentDetailDto>

    suspend fun skills(directory: String): List<SkillDto>

    suspend fun removeSkill(directory: String, location: String): Boolean

    suspend fun removeAgent(directory: String, name: String): Boolean

    suspend fun createAgent(directory: String, input: AgentCreateDto): Boolean

    suspend fun commands(directory: String): List<CommandDto>

    suspend fun mcpStatus(directory: String): List<McpStatusDto>

    suspend fun mcpConfig(directory: String): Map<String, McpServerConfigDto>

    suspend fun saveMcp(directory: String, name: String, scope: String, config: McpConfigDto?): Boolean

    suspend fun mcpConnect(directory: String, name: String): Boolean

    suspend fun mcpDisconnect(directory: String, name: String): Boolean

    suspend fun mcpAuthenticate(directory: String, name: String): Boolean

    suspend fun claudeCodeCompat(): Boolean

    suspend fun setClaudeCodeCompat(value: Boolean): Boolean
}
