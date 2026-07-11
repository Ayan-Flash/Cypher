/**
 * Legacy Cypher CLI migration module
 *
 * Migrates authentication from the legacy Cypher CLI VS Code extension CLI
 * config path (~/.cypher/cli/config.json) to the new auth.json format.
 */
import fs from "fs/promises"
import os from "os"
import path from "path"

export const LEGACY_CONFIG_PATH = path.join(os.homedir(), ".cypher", "cli", "config.json")

interface LegacyProvider {
  id: string
  provider: string
  cypherToken?: string
  cypherModel?: string
  cypherOrganizationId?: string
}

interface LegacyConfig {
  providers?: LegacyProvider[]
}

interface LegacyCypherAuth {
  token: string
  organizationId?: string
}

// Auth info types matching opencode's Auth module
type ApiAuth = { type: "api"; key: string }
type OAuthAuth = { type: "oauth"; access: string; refresh: string; expires: number; accountId?: string }
type AuthInfo = ApiAuth | OAuthAuth

/**
 * Extract cypher auth from legacy config
 */
function extractCypherAuth(config: LegacyConfig): LegacyCypherAuth | undefined {
  if (!config.providers) return undefined

  const provider = config.providers.find((p) => p.provider === "cypher")
  if (!provider?.cypherToken) return undefined

  return {
    token: provider.cypherToken,
    organizationId: provider.cypherOrganizationId,
  }
}

/**
 * Migrate Cypher authentication from legacy CLI config path.
 *
 * Checks ~/.cypher/cli/config.json for existing cypher credentials
 * and migrates them to the new auth.json format.
 *
 * @param hasCypherAuth - Callback to check if cypher auth already exists
 * @param saveCypherAuth - Callback to save the migrated auth
 * @returns true if migration was performed, false otherwise
 */
export async function migrateLegacyCypherAuth(
  hasCypherAuth: () => Promise<boolean>,
  saveCypherAuth: (auth: AuthInfo) => Promise<void>,
): Promise<boolean> {
  // Skip if cypher auth already configured
  if (await hasCypherAuth()) return false

  // Check if legacy config exists and parse it
  const content = await fs.readFile(LEGACY_CONFIG_PATH, "utf-8").catch(() => null)
  if (!content) return false

  let config: LegacyConfig | null = null
  try {
    config = JSON.parse(content) as LegacyConfig
  } catch {
    return false
  }

  // Extract cypher auth from legacy config
  const legacy = extractCypherAuth(config)
  if (!legacy) return false

  // Migrate to new format
  // Use OAuth format if organization ID present, otherwise API format
  if (legacy.organizationId) {
    await saveCypherAuth({
      type: "oauth",
      access: legacy.token,
      refresh: "",
      expires: 0,
      accountId: legacy.organizationId,
    })
  } else {
    await saveCypherAuth({
      type: "api",
      key: legacy.token,
    })
  }

  return true
}
