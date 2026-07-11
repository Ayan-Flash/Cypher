@file:Suppress("UnstableApiUsage")

package ai.cypher.backend.rpc

import ai.cypher.rpc.CypherAppRpcApi
import com.intellij.platform.rpc.backend.RemoteApiProvider
import fleet.rpc.remoteApiDescriptor

internal class CypherAppRpcApiProvider : RemoteApiProvider {
    override fun RemoteApiProvider.Sink.remoteApis() {
        remoteApi(remoteApiDescriptor<CypherAppRpcApi>()) {
            CypherAppRpcApiImpl()
        }
    }
}
