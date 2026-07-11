import { createMemo } from "solid-js"
import { useSync } from "@tui/context/sync"

export function useConnected() {
  const sync = useSync()
  // cypher_change - exclude "cypher" (anonymous autoload) alongside "opencode"
  return createMemo(() =>
    sync.data.provider.some(
      (x) => (x.id !== "opencode" && x.id !== "cypher") || Object.values(x.models).some((y) => y.cost?.input !== 0),
    ),
  )
}
