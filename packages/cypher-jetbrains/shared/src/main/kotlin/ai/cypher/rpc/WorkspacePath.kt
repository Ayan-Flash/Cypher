package ai.cypher.rpc

fun isManagedWorktreeStorage(path: String): Boolean {
    val rel = path.replace('\\', '/').trimStart('/')
    return rel == ".cypher/worktrees" || rel.startsWith(".cypher/worktrees/")
}
