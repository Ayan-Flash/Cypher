@file:Suppress("UnstableApiUsage")

package ai.cypher.backend.rpc

import ai.cypher.rpc.CypherWorkspaceRpcApi
import com.intellij.platform.rpc.backend.RemoteApiProvider
import fleet.rpc.remoteApiDescriptor

internal class CypherProjectRpcApiProvider : RemoteApiProvider {
    override fun RemoteApiProvider.Sink.remoteApis() {
        remoteApi(remoteApiDescriptor<CypherWorkspaceRpcApi>()) {
            CypherWorkspaceRpcApiImpl()
        }
    }
}
