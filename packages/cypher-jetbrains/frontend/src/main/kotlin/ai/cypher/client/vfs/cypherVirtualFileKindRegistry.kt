package ai.cypher.client.vfs

import com.intellij.openapi.components.Service
import java.util.concurrent.ConcurrentHashMap

@Service(Service.Level.APP)
class CypherVirtualFileKindRegistry {
    private val kinds = ConcurrentHashMap<String, CypherVirtualFileKind>()

    fun register(kind: CypherVirtualFileKind) {
        kinds[kind.id] = kind
    }

    fun unregister(id: String) {
        kinds.remove(id)
    }

    fun clear() {
        kinds.clear()
    }

    fun get(id: String): CypherVirtualFileKind? = kinds[id]
}
