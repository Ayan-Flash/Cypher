@file:Suppress("UnstableApiUsage")

package ai.cypher.rpc

import ai.cypher.rpc.dto.CustomModelFetchDto
import ai.cypher.rpc.dto.CustomModelFetchResultDto
import ai.cypher.rpc.dto.CustomProviderSaveDto
import ai.cypher.rpc.dto.ProviderActionResultDto
import ai.cypher.rpc.dto.ProviderConnectDto
import ai.cypher.rpc.dto.ProviderDisconnectDto
import ai.cypher.rpc.dto.ProviderEnableDto
import ai.cypher.rpc.dto.ProviderOAuthAuthorizeDto
import ai.cypher.rpc.dto.ProviderOAuthCallbackDto
import ai.cypher.rpc.dto.ProviderOAuthReadyDto
import ai.cypher.rpc.dto.ProviderSettingsDto
import com.intellij.platform.rpc.RemoteApiProviderService
import fleet.rpc.RemoteApi
import fleet.rpc.Rpc
import fleet.rpc.remoteApiDescriptor

@Rpc
interface CypherProviderRpcApi : RemoteApi<Unit> {
    companion object {
        suspend fun getInstance(): CypherProviderRpcApi {
            return RemoteApiProviderService.resolve(remoteApiDescriptor<CypherProviderRpcApi>())
        }
    }

    suspend fun state(directory: String): ProviderSettingsDto
    suspend fun connect(input: ProviderConnectDto): ProviderActionResultDto
    suspend fun authorize(input: ProviderOAuthAuthorizeDto): ProviderOAuthReadyDto
    suspend fun callback(input: ProviderOAuthCallbackDto): ProviderActionResultDto
    suspend fun disconnect(input: ProviderDisconnectDto): ProviderActionResultDto
    suspend fun enable(input: ProviderEnableDto): ProviderActionResultDto
    suspend fun saveCustom(input: CustomProviderSaveDto): ProviderActionResultDto
    suspend fun fetchCustomModels(input: CustomModelFetchDto): CustomModelFetchResultDto
}
