import type { CypherClient } from "@cypher/sdk/v2/client"

export async function stopSessionProcesses(
  client: CypherClient | null,
  sessionID: string,
  directory: string,
): Promise<void> {
  if (!client) return
  await client.backgroundProcess
    .stopSession({ sessionID, directory })
    .catch((err: unknown) => console.warn("[Cypher New] CypherProvider: Failed to stop background processes:", err))
}
