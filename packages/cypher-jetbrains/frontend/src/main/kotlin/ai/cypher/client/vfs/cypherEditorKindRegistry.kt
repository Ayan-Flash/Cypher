package ai.cypher.client.vfs

import com.intellij.openapi.components.Service
import com.intellij.openapi.components.service
import java.util.concurrent.ConcurrentHashMap

@Service(Service.Level.APP)
class CypherEditorKindRegistry {
    private val kinds = ConcurrentHashMap<String, CypherEditorKind>()

    fun register(kind: CypherEditorKind) {
        kinds[kind.id] = kind
        service<CypherVirtualFileKindRegistry>().register(kind)
    }

    fun unregister(id: String) {
        kinds.remove(id)
        service<CypherVirtualFileKindRegistry>().unregister(id)
    }

    fun clear() {
        kinds.keys.forEach { id -> unregister(id) }
    }

    fun get(id: String): CypherEditorKind? = kinds[id]
}
