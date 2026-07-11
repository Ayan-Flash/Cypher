// cypher_change - new file
// Cypher-specific overrides for the server control plane.
// Imported by ../../server/server.ts with minimal cypher_change markers.

/** Additional CORS origin check for *.cypher.ai */
export function corsOrigin(input: string): string | undefined {
  if (/^https:\/\/([a-z0-9-]+\.)*cypher\.ai$/.test(input)) {
    return input
  }
  return undefined
}

export const DOC_TITLE = "cypher"
export const DOC_DESCRIPTION = "cypher api"
