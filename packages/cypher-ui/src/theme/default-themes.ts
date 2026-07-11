import type { DesktopTheme } from "@opencode-ai/ui/theme/types"
import { DEFAULT_THEMES as UPSTREAM_THEMES } from "@opencode-ai/ui/theme/default-themes"
import cypherJson from "./themes/cypher.json"
import cypherVscodeJson from "./themes/cypher-vscode.json"

// Re-export all upstream theme constants
export {
  oc2Theme,
  tokyonightTheme,
  draculaTheme,
  monokaiTheme,
  solarizedTheme,
  nordTheme,
  catppuccinTheme,
  ayuTheme,
  oneDarkProTheme,
  shadesOfPurpleTheme,
  nightowlTheme,
  vesperTheme,
  carbonfoxTheme,
  gruvboxTheme,
  auraTheme,
} from "@opencode-ai/ui/theme/default-themes"

export const cypherTheme = cypherJson as DesktopTheme
export const cypherVscodeTheme = cypherVscodeJson as DesktopTheme

export const CYPHER_THEMES: Record<string, DesktopTheme> = {
  cypher: cypherTheme,
  "cypher-vscode": cypherVscodeTheme,
}

// Override DEFAULT_THEMES: Cypher themes first, then upstream
export const DEFAULT_THEMES: Record<string, DesktopTheme> = {
  ...CYPHER_THEMES,
  ...UPSTREAM_THEMES,
}
