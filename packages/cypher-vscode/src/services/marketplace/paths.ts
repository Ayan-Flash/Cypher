import * as path from "path"
import * as os from "os"

/**
 * Global config dir: ~/.config/cypher/ (XDG_CONFIG_HOME/cypher)
 * This matches where the CLI reads global config from.
 */
function globalConfigDir(): string {
  const xdg = process.env.XDG_CONFIG_HOME || path.join(os.homedir(), ".config")
  return path.join(xdg, "cypher")
}

export class MarketplacePaths {
  /** Project-scope config file: <workspace>/.cypher/cypher.json */
  configPath(scope: "project" | "global", workspace?: string): string {
    if (scope === "project") return path.join(workspace!, ".cypher", "cypher.json")
    return path.join(globalConfigDir(), "cypher.json")
  }

  /** Agent install directory (where marketplace agents are written as .md files). */
  agentsDir(scope: "project" | "global", workspace?: string): string {
    if (scope === "project") return path.join(workspace!, ".cypher", "agents")
    return path.join(globalConfigDir(), "agents")
  }

  /** Skill install directory (where the marketplace installer writes to). */
  skillsDir(scope: "project" | "global", workspace?: string): string {
    if (scope === "project") return path.join(workspace!, ".cypher", "skills")
    return path.join(os.homedir(), ".cypher", "skills")
  }
}
