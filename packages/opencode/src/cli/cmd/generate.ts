import { Server } from "../../server/server"
import type { CommandModule } from "yargs"

type Args = {}

export const GenerateCommand = {
  command: "generate",
  builder: (yargs) => yargs,
  handler: async () => {
    const specs = (await Server.openapi()) as {
      info: { title: string; description: string } // cypher_change
      paths: Record<string, Record<string, any>>
    }
    // cypher_change start
    specs.info.title = "cypher"
    specs.info.description = "cypher api"
    // cypher_change end
    for (const item of Object.values(specs.paths)) {
      for (const method of ["get", "post", "put", "delete", "patch"] as const) {
        const operation = item[method]
        if (!operation?.operationId) continue
        operation["x-codeSamples"] = [
          // cypher_change start
          {
            lang: "js",
            source: [
              `import { createCypherClient } from "@cypher/sdk"`,
              ``,
              `const client = createCypherClient()`,
              `await client.${operation.operationId}({`,
              `  ...`,
              `})`,
            ].join("\n"),
          },
          // cypher_change end,
        ]
      }
    }
    const raw = JSON.stringify(specs, null, 2)
      // cypher_change start - replace upstream product name in all descriptions
      .replaceAll("OpenCode", "Cypher")
      .replaceAll("opencode.local", "cypher.local")
      .replaceAll("opencode serve", "cypher serve")
      .replaceAll("https://opencode.ai/", "https://cypher.ai/")
    // cypher_change end

    // Format through prettier so output is byte-identical to committed file
    // regardless of whether ./script/format.ts runs afterward.
    const prettier = await import("prettier")
    const babel = await import("prettier/plugins/babel")
    const estree = await import("prettier/plugins/estree")
    const format = prettier.format ?? prettier.default?.format
    const json = await format(raw, {
      parser: "json",
      plugins: [babel.default ?? babel, estree.default ?? estree],
      printWidth: 120,
    })

    // Wait for stdout to finish writing before process.exit() is called
    await new Promise<void>((resolve, reject) => {
      process.stdout.write(json, (err) => {
        if (err) reject(err)
        else resolve()
      })
    })
  },
} satisfies CommandModule<object, Args>
