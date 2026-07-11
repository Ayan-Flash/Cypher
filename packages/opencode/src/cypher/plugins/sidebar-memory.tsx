import type { TuiPlugin } from "@cypher/plugin/tui"
import type { InternalTuiPlugin } from "@/cli/cmd/tui/plugin/internal"
import { MemorySidebar } from "@/cypher/cli/cmd/tui/component/memory-sidebar"

const id = "internal:cypher-sidebar-memory"

const tui: TuiPlugin = async (api) => {
  api.slots.register({
    order: 1000,
    slots: {
      sidebar_content(_ctx, props) {
        return <MemorySidebar api={api} sessionID={props.session_id} />
      },
    },
  })
}

const plugin: InternalTuiPlugin = {
  id,
  tui,
}

export default plugin
