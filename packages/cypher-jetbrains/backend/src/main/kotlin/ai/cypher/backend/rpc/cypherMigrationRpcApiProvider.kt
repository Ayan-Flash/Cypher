@file:Suppress("UnstableApiUsage")

package ai.cypher.backend.rpc

import ai.cypher.rpc.CypherMigrationRpcApi
import com.intellij.platform.rpc.backend.RemoteApiProvider
import fleet.rpc.remoteApiDescriptor

internal class CypherMigrationRpcApiProvider : RemoteApiProvider {
    override fun RemoteApiProvider.Sink.remoteApis() {
        remoteApi(remoteApiDescriptor<CypherMigrationRpcApi>()) {
            CypherMigrationRpcApiImpl()
        }
    }
}
