@file:Suppress("UnstableApiUsage")

package ai.cypher.backend.rpc

import ai.cypher.rpc.CypherAgentBehaviorRpcApi
import com.intellij.platform.rpc.backend.RemoteApiProvider
import fleet.rpc.remoteApiDescriptor

internal class CypherAgentBehaviorRpcApiProvider : RemoteApiProvider {
    override fun RemoteApiProvider.Sink.remoteApis() {
        remoteApi(remoteApiDescriptor<CypherAgentBehaviorRpcApi>()) {
            CypherAgentBehaviorRpcApiImpl()
        }
    }
}
