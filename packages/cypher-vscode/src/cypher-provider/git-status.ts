import type { CypherClient } from "@cypher/sdk/v2/client"

export async function hasGit(client: CypherClient, directory: string): Promise<boolean> {
  return client.project
    .current({ directory })
    .then((r) => r.data?.vcs === "git")
    .catch(() => false)
}
