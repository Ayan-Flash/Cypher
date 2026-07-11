import { InstallationVersion } from "@opencode-ai/core/installation/version"

export const DEFAULT_HEADERS = {
  "HTTP-Referer": "https://cypher.ai",
  "X-Title": "Cypher CLI",
  "User-Agent": `Cypher-Code/${InstallationVersion}`,
}
