import type { TuiPlugin } from "@cypher/plugin/tui"
import type { InternalTuiPlugin } from "@/cli/cmd/tui/plugin/internal"
import { MemoryPermission } from "@/cypher/cli/cmd/tui/permissions"

const id = "internal:cypher-permissions"

const tui: TuiPlugin = async () => {
  MemoryPermission.register()
}

const plugin: InternalTuiPlugin = {
  id,
  tui,
}

export default plugin
