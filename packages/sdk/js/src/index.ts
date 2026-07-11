export * from "./client.js"
export * from "./server.js"

import { createCypherClient } from "./client.js"
import { createCypherServer } from "./server.js"
import type { ServerOptions } from "./server.js"

export async function createCypher(options?: ServerOptions) {
  const server = await createCypherServer({
    ...options,
  })

  const client = createCypherClient({
    baseUrl: server.url,
  })

  return {
    client,
    server,
  }
}
