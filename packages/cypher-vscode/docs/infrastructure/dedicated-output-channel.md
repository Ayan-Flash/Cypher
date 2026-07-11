# Dedicated Output Channel

**Priority:** P2

Agent Manager has its own output channel. No general "Cypher CLI" output channel exists.

## Remaining Work

- Create `vscode.window.createOutputChannel("Cypher CLI")` during activation
- Centralized logging utility with log levels (debug, info, warn, error)
- Route all `[Cypher New]` log messages to this channel
- Dispose on deactivation
- Migrate existing `console.log("[Cypher New] ...")` calls to the logger
