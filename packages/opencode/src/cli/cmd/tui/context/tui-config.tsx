// cypher_change start - reactive TUI config provider enables hot reload (impl in cypher mirror)
import { CypherTuiConfig } from "@/cypher/cli/cmd/tui/context/tui-config"

export const useTuiConfig = CypherTuiConfig.use
export const TuiConfigProvider = CypherTuiConfig.Provider
// cypher_change end
