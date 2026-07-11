@file:Suppress("UnstableApiUsage")

package ai.cypher.backend.rpc

import ai.cypher.rpc.CypherProviderRpcApi
import com.intellij.platform.rpc.backend.RemoteApiProvider
import fleet.rpc.remoteApiDescriptor

internal class CypherProviderRpcApiProvider : RemoteApiProvider {
    override fun RemoteApiProvider.Sink.remoteApis() {
        remoteApi(remoteApiDescriptor<CypherProviderRpcApi>()) {
            CypherProviderRpcApiImpl()
        }
    }
}
