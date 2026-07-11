import HomeFooter from "../feature-plugins/home/footer"
import HomeTips from "../feature-plugins/home/tips"
// cypher_change start
import HomeNews from "@/cypher/plugins/home-news"
import HomeOnboarding from "@/cypher/plugins/home-onboarding"
import CypherAttention from "@/cypher/plugins/attention"
import CypherHomeFooter from "@/cypher/plugins/home-footer"
import CypherPermissions from "@/cypher/plugins/permissions" // cypher_change
import CypherSidebarFooter from "@/cypher/plugins/sidebar-footer"
import CypherSidebarBackgroundProcesses from "@/cypher/plugins/sidebar-background-processes"
import CypherSidebarIndexing from "@/cypher/plugins/sidebar-indexing"
import CypherSidebarMemory from "@/cypher/plugins/sidebar-memory" // cypher_change
import CypherMemoryPalette from "@/cypher/plugins/memory-palette" // cypher_change
import CypherSidebarPr from "@/cypher/plugins/sidebar-pr"
import CypherSidebarUsage from "@/cypher/plugins/sidebar-usage"
import CypherSandbox from "@/cypher/plugins/sandbox"
import CypherRemote from "@/cypher/plugins/remote"
import CypherReload from "@/cypher/plugins/reload"
// cypher_change end
import SidebarContext from "../feature-plugins/sidebar/context"
import SidebarMcp from "../feature-plugins/sidebar/mcp"
import SidebarLsp from "../feature-plugins/sidebar/lsp"
import SidebarTodo from "../feature-plugins/sidebar/todo"
import SidebarFiles from "../feature-plugins/sidebar/files"
import SidebarFooter from "../feature-plugins/sidebar/footer"
import PluginManager from "../feature-plugins/system/plugins"
import Notifications from "../feature-plugins/system/notifications"
import SessionV2Debug from "../feature-plugins/system/session-v2"
import WhichKey from "../feature-plugins/system/which-key"
import DiffViewer from "../feature-plugins/system/diff-viewer"
import SessionSwitcher from "../feature-plugins/session"
import { Flag } from "@opencode-ai/core/flag/flag"
import type { TuiPlugin, TuiPluginModule } from "@cypher/plugin/tui"
import type { RuntimeFlags } from "@/effect/runtime-flags"

export type InternalTuiPlugin = Omit<TuiPluginModule, "id"> & {
  id: string
  tui: TuiPlugin
  enabled?: boolean
}

export function internalTuiPlugins(flags: Pick<RuntimeFlags.Info, "experimentalEventSystem">): InternalTuiPlugin[] {
  return [
    HomeNews, // cypher_change
    HomeOnboarding, // cypher_change
    CypherAttention, // cypher_change
    CypherHomeFooter, // cypher_change
    CypherPermissions, // cypher_change
    CypherSidebarFooter, // cypher_change
    CypherSidebarMemory, // cypher_change
    CypherMemoryPalette, // cypher_change
    CypherSidebarBackgroundProcesses, // cypher_change
    CypherSidebarIndexing, // cypher_change
    CypherSidebarPr, // cypher_change
    CypherSidebarUsage, // cypher_change
    CypherSandbox, // cypher_change
    CypherRemote, // cypher_change
    CypherReload, // cypher_change
    HomeFooter,
    HomeTips,
    SidebarContext,
    SidebarMcp,
    SidebarLsp,
    SidebarTodo,
    SidebarFiles,
    SidebarFooter,
    Notifications,
    PluginManager,
    WhichKey,
    DiffViewer,
    ...(flags.experimentalEventSystem ? [SessionV2Debug] : []),
    ...(Flag.CYPHER_EXPERIMENTAL_SESSION_SWITCHER ? [SessionSwitcher] : []),
  ]
}
