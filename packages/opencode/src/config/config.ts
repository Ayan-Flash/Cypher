import * as Log from "@opencode-ai/core/util/log"
import { serviceUse } from "@opencode-ai/core/effect/service-use"
import path from "path"
import { pathToFileURL } from "url"
import os from "os"
import { mergeDeep } from "remeda"
import { Global } from "@opencode-ai/core/global"
import fsNode from "fs/promises"
import { NamedError } from "@opencode-ai/core/util/error"
import { Flag } from "@opencode-ai/core/flag/flag"
import { Auth } from "../auth"
import { Env } from "../env"
import { applyEdits, findNodeAtLocation, modify, parseTree } from "jsonc-parser" // cypher_change - parseTree/findNodeAtLocation used in patchJsonc
import { InstallationLocal, InstallationVersion } from "@opencode-ai/core/installation/version"
import { existsSync } from "fs"
// cypher_change start
import { GlobalBus } from "@/bus/global"
import { Event } from "../server/event"
// cypher_change end
import { Account } from "@/account/account"
import { isRecord } from "@/util/record"
import type { ConsoleState } from "./console-state"
import { AppFileSystem } from "@opencode-ai/core/filesystem"
import { InstanceState } from "@/effect/instance-state"
import { Context, Duration, Effect, Exit, Fiber, Layer, Option, Schema } from "effect"
import { FetchHttpClient, HttpClient, HttpClientRequest, HttpClientResponse } from "effect/unstable/http"
import { EffectFlock } from "@opencode-ai/core/util/effect-flock"
import { containsPath, type InstanceContext } from "../project/instance-context"
import { NonNegativeInt, PositiveInt, type DeepMutable } from "@opencode-ai/core/schema"
import { ConfigAgent } from "./agent"
import { ConfigAttachment } from "./attachment"
import { ConfigCommand } from "./command"
import { ConfigFormatter } from "./formatter"
import { ConfigLayout } from "./layout"
import { ConfigLSP } from "./lsp"
import { ConfigManaged } from "./managed"
import { ConfigMCP } from "./mcp"
import { ConfigModelID } from "./model-id"
import { ConfigParse } from "./parse"
import { ConfigPaths } from "./paths"
import { ConfigPermission } from "./permission"
import { ConfigPlugin } from "./plugin"
import { ConfigProvider } from "./provider"
import { ConfigReference } from "./reference"
import { ConfigServer } from "./server"
import { ConfigSkills } from "./skills"
import { ConfigVariable } from "./variable"
import { Npm } from "@opencode-ai/core/npm"
import z from "zod" // cypher_change - Cypher config compatibility schemas
// cypher_change start
import { ZodOverride } from "@opencode-ai/core/effect-zod"
import { CyphercodeConfig } from "../cypher/config/config"
import { primaryPaths } from "../cypher/primary-worktree"
import { Git } from "@/git"
import { CyphercodeDefaultPlugins } from "@/cypher/config/default-plugins"
import { CyphercodeGlobalConfigStamp } from "@/cypher/config/global-stamp"
import { SandboxConfig } from "@/cypher/sandbox/config"
import {
  IndexingConfig as CypherIndexingConfig,
  IndexingSchema as CypherIndexingSchema,
} from "@cypher/cypher-indexing/config"
import { unique } from "remeda"
// cypher_change end
import { withTransientReadRetry } from "@/util/effect-http-client"
import { ConfigExperimental } from "@opencode-ai/core/config/experimental"

const log = Log.create({ service: "config" })

// Custom merge function that concatenates array fields instead of replacing them
// Keep remeda's deep conditional merge type out of hot config-loading paths; TS profiling showed it dominates here.
function mergeConfig(target: Info, source: Info): Info {
  return mergeDeep(target, source) as Info
}

function mergeConfigConcatArrays(target: Info, source: Info): Info {
  const merged = mergeConfig(target, source)
  if (target.instructions && source.instructions) {
    merged.instructions = Array.from(new Set([...target.instructions, ...source.instructions]))
  }
  return merged
}

function normalizeLoadedConfig(data: unknown, source: string) {
  if (!isRecord(data)) return data
  const copy = CyphercodeConfig.retireIndexingFlag({ ...data }, source) // cypher_change
  const hadLegacy = "theme" in copy || "keybinds" in copy || "tui" in copy
  if (!hadLegacy) return copy
  delete copy.theme
  delete copy.keybinds
  delete copy.tui
  log.warn("tui keys in opencode config are deprecated; move them to tui.json", { path: source })
  return copy
}

// cypher_change start
export const Warning = z.object({
  path: z.string(),
  message: z.string(),
  detail: z.string().optional(),
})
export type Warning = z.infer<typeof Warning>

const { caught: caughtWarning } = CyphercodeConfig
// cypher_change end

async function substituteWellKnownRemoteConfig(input: {
  value: unknown
  dir: string
  source: string
  env: Record<string, string>
}) {
  if (!isRecord(input.value) || typeof input.value.url !== "string") return undefined

  const url = await ConfigVariable.substitute({
    text: input.value.url,
    type: "virtual",
    dir: input.dir,
    source: input.source,
    env: input.env,
    trusted: true, // cypher_change - well-known org config is a trusted source
  })
  const headers = isRecord(input.value.headers)
    ? Object.fromEntries(
        await Promise.all(
          Object.entries(input.value.headers)
            .filter((entry): entry is [string, string] => typeof entry[1] === "string")
            .map(async ([key, value]) => [
              key,
              await ConfigVariable.substitute({
                text: value,
                type: "virtual",
                dir: input.dir,
                source: input.source,
                env: input.env,
                trusted: true, // cypher_change - well-known org config is a trusted source
              }),
            ]),
        ),
      )
    : undefined

  return { url, headers }
}

const WellKnownConfig = Schema.Struct({
  config: Schema.optional(Schema.Json),
  remote_config: Schema.optional(Schema.Json),
})

async function resolveLoadedPlugins<T extends { plugin?: ConfigPlugin.Spec[] }>(config: T, filepath: string) {
  if (!config.plugin) return config
  for (let i = 0; i < config.plugin.length; i++) {
    // Normalize path-like plugin specs while we still know which config file declared them.
    // This prevents `./plugin.ts` from being reinterpreted relative to some later merge location.
    config.plugin[i] = await ConfigPlugin.resolvePluginSpec(config.plugin[i], filepath)
  }
  return config
}

export type Layout = ConfigLayout.Layout

// cypher_change start - indexing configuration
export const Indexing = CypherIndexingConfig
export type Indexing = z.infer<typeof Indexing>
// cypher_change end

const LogLevelRef = Schema.Literals(["DEBUG", "INFO", "WARN", "ERROR"]).annotate({
  identifier: "LogLevel",
  description: "Log level",
})
const Percent = Schema.Number.check(Schema.isGreaterThan(0), Schema.isLessThanOrEqualTo(100)) // cypher_change

const IndexingRef = CypherIndexingSchema.annotate({ [ZodOverride]: CypherIndexingConfig }) // cypher_change

export const Info = Schema.Struct({
  $schema: Schema.optional(Schema.String).annotate({
    description: "JSON schema reference for configuration validation",
  }),
  shell: Schema.optional(Schema.String).annotate({
    description: "Default shell to use for terminal and bash tool",
  }),
  logLevel: Schema.optional(LogLevelRef).annotate({ description: "Log level" }),
  server: Schema.optional(ConfigServer.Server).annotate({
    description: "Server configuration for the cypher serve command", // cypher_change
  }),
  command: Schema.optional(Schema.Record(Schema.String, ConfigCommand.Info)).annotate({
    description: "Command configuration, see https://cypher.ai/docs/customize/workflows", // cypher_change
  }),
  skills: Schema.optional(ConfigSkills.Info).annotate({ description: "Additional skill folder paths" }),
  reference: Schema.optional(ConfigReference.Info).annotate({
    description: "Named git or local directory references that can be mentioned as @alias or @alias/path",
  }),
  watcher: Schema.optional(
    Schema.Struct({
      ignore: Schema.optional(Schema.mutable(Schema.Array(Schema.String))),
    }),
  ),
  snapshot: Schema.optional(Schema.Boolean).annotate({
    description:
      "Enable or disable snapshot tracking. When false, filesystem snapshots are not recorded and undoing or reverting will not undo/redo file changes. Defaults to true.",
  }),
  // User-facing plugin config is stored as Specs; provenance gets attached later while configs are merged.
  plugin: Schema.optional(Schema.mutable(Schema.Array(ConfigPlugin.Spec))),
  share: Schema.optional(Schema.Literals(["manual", "auto", "disabled"])).annotate({
    description:
      "Control sharing behavior:'manual' allows manual sharing via commands, 'auto' enables automatic sharing, 'disabled' disables all sharing",
  }),
  autoshare: Schema.optional(Schema.Boolean).annotate({
    description: "@deprecated Use 'share' field instead. Share newly created sessions automatically",
  }),
  autoupdate: Schema.optional(Schema.Union([Schema.Boolean, Schema.Literal("notify")])).annotate({
    description:
      "Automatically update to the latest version. Set to true to auto-update, false to disable, or 'notify' to show update notifications",
  }),
  disabled_providers: Schema.optional(Schema.mutable(Schema.Array(Schema.String))).annotate({
    description: "Disable providers that are loaded automatically",
  }),
  enabled_providers: Schema.optional(Schema.mutable(Schema.Array(Schema.String))).annotate({
    description: "When set, ONLY these providers will be enabled. All other providers will be ignored",
  }),
  // cypher_change start
  // NOTE: Any new cypher_change key added to Config.Info must also be mirrored in
  // apps/web/src/app/config.json/extras.ts in the cloud repo, otherwise
  // $schema: https://app.cypher.ai/config.json will not recognize it.
  remote_control: Schema.optional(Schema.Boolean).annotate({
    description: "Enable remote control of sessions via Cypher Cloud. Equivalent to running /remote on startup.",
  }),
  auto_collapse_reasoning: Schema.optional(Schema.Boolean).annotate({
    description: "Automatically collapse reasoning blocks after the agent finishes writing them",
  }),
  indexing: Schema.optional(IndexingRef).annotate({ description: "Codebase indexing configuration" }),
  console: Schema.optional(
    Schema.Struct({
      context_sidebar_width: Schema.optional(
        Schema.Int.check(Schema.isBetween({ minimum: 250, maximum: 800 })).annotate({
          description: "Width of the Cypher Console project context sidebar in pixels",
        }),
      ),
      diff_style: Schema.optional(Schema.Literals(["unified", "split"])).annotate({
        description: "Default diff layout in Cypher Console project reviews",
      }),
    }),
  ).annotate({ description: "Cypher Console user interface configuration" }),
  terminal_command_display: Schema.optional(Schema.Literals(["expanded", "collapsed"])).annotate({
    description: "Controls whether terminal command blocks are expanded or collapsed by default in the VS Code chat UI",
  }),
  code_edit_display: Schema.optional(Schema.Literals(["expanded", "collapsed"])).annotate({
    description:
      "Controls whether code edit and diff blocks are expanded or collapsed by default in the VS Code chat UI",
  }),
  hide_prompt_training_models: Schema.optional(Schema.Boolean).annotate({
    description: "Hide Cypher Gateway models that may train on your prompts from model listings",
  }),
  sandbox: Schema.optional(SandboxConfig.Info),
  model: Schema.optional(Schema.NullOr(ConfigModelID)).annotate({
    description: "Model to use in the format of provider/model, eg anthropic/claude-2",
  }),
  small_model: Schema.optional(Schema.NullOr(ConfigModelID)).annotate({
    description: "Small model to use for tasks like title generation in the format of provider/model",
  }),
  subagent_model: Schema.optional(Schema.NullOr(ConfigModelID)).annotate({
    description:
      "Default model for task-tool subagents in the format of provider/model. If unset or unavailable, subagents inherit the calling agent model.",
  }),
  subagent_variant: Schema.optional(Schema.NullOr(Schema.String)).annotate({
    description: "Default model variant for task-tool subagents when subagent_model is configured.",
  }),
  subagent_variant_overrides: Schema.optional(
    Schema.NullOr(Schema.Record(Schema.String, Schema.NullOr(Schema.String))),
  ).annotate({
    description:
      "Model-specific variant overrides for task-tool subagents, keyed by provider/model. Valid overrides take precedence over saved, agent-specific, and inherited variants.",
  }),
  default_agent: Schema.optional(Schema.NullOr(Schema.String)).annotate({
    description:
      "Default agent to use when none is specified. Must be a primary agent. Falls back to 'code' if not set or if the specified agent is invalid.",
  }),
  // cypher_change end
  username: Schema.optional(Schema.String).annotate({
    description: "Custom username to display in conversations instead of system username",
  }),
  mode: Schema.optional(
    Schema.StructWithRest(
      Schema.Struct({
        build: Schema.optional(ConfigAgent.Info),
        plan: Schema.optional(ConfigAgent.Info),
      }),
      [Schema.Record(Schema.String, ConfigAgent.Info)],
    ),
  ).annotate({ description: "@deprecated Use `agent` field instead." }),
  agent: Schema.optional(
    Schema.StructWithRest(
      Schema.Struct({
        // primary
        plan: Schema.optional(ConfigAgent.Info),
        build: Schema.optional(ConfigAgent.Info),
        // cypher_change start
        debug: Schema.optional(ConfigAgent.Info),
        orchestrator: Schema.optional(ConfigAgent.Info),
        ask: Schema.optional(ConfigAgent.Info),
        // cypher_change end
        // subagent
        general: Schema.optional(ConfigAgent.Info),
        explore: Schema.optional(ConfigAgent.Info),
        scout: Schema.optional(ConfigAgent.Info),
        // specialized
        title: Schema.optional(ConfigAgent.Info),
        summary: Schema.optional(ConfigAgent.Info),
        compaction: Schema.optional(ConfigAgent.Info),
      }),
      [Schema.Record(Schema.String, ConfigAgent.Info)],
    ),
    // cypher_change start
  ).annotate({ description: "Agent configuration, see https://cypher.ai/docs/customize/custom-subagents" }), // cypher_change
  provider: Schema.optional(Schema.Record(Schema.String, Schema.NullOr(ConfigProvider.Info))).annotate({
    // cypher_change end
    description: "Custom provider configurations and model overrides",
  }),
  mcp: Schema.optional(
    Schema.Record(
      Schema.String,
      Schema.Union([
        ConfigMCP.Info,
        // Matches the legacy `{ enabled: false }` form used to disable a server.
        Schema.Struct({ enabled: Schema.Boolean }),
      ]),
    ),
  ).annotate({ description: "MCP (Model Context Protocol) server configurations" }),
  formatter: Schema.optional(ConfigFormatter.Info).annotate({
    description:
      "Enable or configure formatters. Omit or set to false to disable, true to enable built-ins, or an object to enable built-ins with overrides.",
  }),
  lsp: Schema.optional(ConfigLSP.Info).annotate({
    description:
      "Enable or configure LSP servers. Omit or set to false to disable, true to enable built-ins, or an object to enable built-ins with overrides.",
  }),
  instructions: Schema.optional(Schema.mutable(Schema.Array(Schema.String))).annotate({
    description: "Additional instruction files or patterns to include",
  }),
  layout: Schema.optional(ConfigLayout.Layout).annotate({ description: "@deprecated Always uses stretch layout." }),
  permission: Schema.optional(ConfigPermission.Info),
  tools: Schema.optional(Schema.Record(Schema.String, Schema.Boolean)),
  attachment: Schema.optional(ConfigAttachment.Info).annotate({
    description: "Attachment processing configuration, including image size limits and resizing behavior",
  }),
  enterprise: Schema.optional(
    Schema.Struct({
      url: Schema.optional(Schema.String).annotate({ description: "Enterprise URL" }),
    }),
  ),
  commit_message: CyphercodeConfig.CommitMessageSchema, // cypher_change
  tool_output: Schema.optional(
    Schema.Struct({
      max_lines: Schema.optional(PositiveInt).annotate({
        description: "Maximum lines of tool output before it is truncated and saved to disk (default: 2000)",
      }),
      max_bytes: Schema.optional(PositiveInt).annotate({
        description: "Maximum bytes of tool output before it is truncated and saved to disk (default: 51200)",
      }),
    }),
  ).annotate({
    description:
      "Thresholds for truncating tool output. When output exceeds either limit, the full text is written to the truncation directory and a preview is returned.",
  }),
  compaction: Schema.optional(
    Schema.Struct({
      auto: Schema.optional(Schema.Boolean).annotate({
        description: "Enable automatic compaction when context is full (default: true)",
      }),
      // cypher_change start
      threshold_percent: Schema.optional(Schema.NullOr(Percent)).annotate({
        description:
          "Percentage of the model input/context window that triggers automatic compaction. The reserved safety buffer still applies if it would compact sooner.",
      }),
      // cypher_change end
      prune: Schema.optional(Schema.Boolean).annotate({
        description: "Enable pruning of old tool outputs (default: true)",
      }),
      tail_turns: Schema.optional(NonNegativeInt).annotate({
        description:
          "Number of recent user turns, including their following assistant/tool responses, to keep verbatim during compaction (default: 2)",
      }),
      preserve_recent_tokens: Schema.optional(NonNegativeInt).annotate({
        description: "Maximum number of tokens from recent turns to preserve verbatim after compaction",
      }),
      reserved: Schema.optional(NonNegativeInt).annotate({
        description: "Token buffer for compaction. Leaves enough window to avoid overflow during compaction.",
      }),
    }),
  ),
  experimental: Schema.optional(
    Schema.Struct({
      disable_paste_summary: Schema.optional(Schema.Boolean),
      batch_tool: Schema.optional(Schema.Boolean).annotate({ description: "Enable the batch tool" }),
      // cypher_change start
      codebase_search: Schema.optional(Schema.Boolean).annotate({ description: "Enable AI-powered codebase search" }),
      image_generation: Schema.optional(Schema.Boolean).annotate({ description: "Enable AI image generation" }),
      image_generation_model: Schema.optional(Schema.String).annotate({
        description: "Model ID to use for image generation (default: openrouter/auto)",
      }),
      agent_requirements: Schema.optional(Schema.Boolean).annotate({
        description: "Require declared agent skills, MCPs, and VS Code extensions before VS Code prompts can run",
      }),
      native_notebook_tools: Schema.optional(Schema.Boolean).annotate({
        description: "Enable native tools for reading, editing, and executing VS Code notebooks",
      }),
      speech_to_text_model: Schema.optional(Schema.String).annotate({
        description: "Speech-to-text transcription model ID to use for voice input",
      }),
      openTelemetry: Schema.Boolean.pipe(Schema.optional, Schema.withDecodingDefault(Effect.succeed(true))).annotate({
        description: "Enable telemetry. Set to false to opt-out.",
      }),
      // cypher_change end
      primary_tools: Schema.optional(Schema.mutable(Schema.Array(Schema.String))).annotate({
        description: "Tools that should only be available to primary agents.",
      }),
      continue_loop_on_deny: Schema.optional(Schema.Boolean).annotate({
        description: "Continue the agent loop when a tool call is denied",
      }),
      // cypher_change start
      swe_pruner: Schema.optional(Schema.Boolean).annotate({
        description:
          "Enable SWE-Pruner: task-aware pruning of large read, grep, and bash tool outputs guided by a focus question provided by the agent (default: false)",
      }),
      swe_pruner_model: Schema.optional(Schema.String).annotate({
        description:
          'Model used by SWE-Pruner to skim tool outputs, in "provider/model" format (default: the configured small model)',
      }),
      // cypher_change end
      mcp_timeout: Schema.optional(PositiveInt).annotate({
        description: "Timeout in milliseconds for model context protocol (MCP) requests",
      }),
      policies: Schema.optional(Schema.mutable(Schema.Array(ConfigExperimental.Policy))).annotate({
        description: "Policy statements applied to supported resources, such as provider access",
      }),
    }),
  ),
}).annotate({ identifier: "Config" })

// Uses the shared `DeepMutable` from `@opencode-ai/core/schema`. See the definition
// there for why the local variant is needed over `Types.DeepMutable` from
// effect-smol (the upstream version collapses `unknown` to `{}`).
export type Info = DeepMutable<Schema.Schema.Type<typeof Info>> & {
  // plugin_origins is derived state, not a persisted config field. It keeps each winning plugin spec together
  // with the file and scope it came from so later runtime code can make location-sensitive decisions.
  plugin_origins?: ConfigPlugin.Origin[]
}

type State = {
  config: Info
  directories: string[]
  deps: Fiber.Fiber<void>[]
  warnings: Warning[] // cypher_change
  consoleState: ConsoleState
}

export interface Interface {
  readonly get: () => Effect.Effect<Info>
  readonly getGlobal: () => Effect.Effect<Info>
  readonly getConsoleState: () => Effect.Effect<ConsoleState>
  readonly update: (config: Info) => Effect.Effect<void>
  // cypher_change start
  readonly updateGlobal: (
    config: Info,
    options?: { dispose?: boolean },
  ) => Effect.Effect<{ info: Info; changed: boolean }>
  // cypher_change end
  readonly invalidate: () => Effect.Effect<void>
  readonly directories: () => Effect.Effect<string[]>
  readonly waitForDependencies: () => Effect.Effect<void>
  readonly warnings: () => Effect.Effect<Warning[]> // cypher_change
}

export class Service extends Context.Service<Service, Interface>()("@opencode/Config") {}

export const use = serviceUse(Service)

function globalConfigFile() {
  // cypher_change start
  const candidates = ["cypher.jsonc", "cypher.json", "opencode.jsonc", "opencode.json", "config.json"].map((file) =>
    // cypher_change end
    path.join(Global.Path.config, file),
  )
  for (const file of candidates) {
    if (existsSync(file)) return file
  }
  return candidates[0]
}

function patchJsonc(input: string, patch: unknown, path: string[] = []): string {
  if (!isRecord(patch)) {
    const edits = modify(input, path, patch === null ? undefined : patch, {
      // cypher_change
      formattingOptions: {
        insertSpaces: true,
        tabSize: 2,
      },
    })
    return applyEdits(input, edits)
  }

  // cypher_change start — when the existing JSONC node at this path is a
  // scalar (e.g. permission.bash is "ask" as a string), jsonc-parser cannot
  // add child keys to it. Detect this case and replace the whole node with
  // the patch object in a single modify() call instead of recursing.
  // For permission keys, promote the scalar to { "*": scalarValue } so the
  // wildcard default is preserved. For other keys, replace directly.
  if (path.length > 0) {
    const tree = parseTree(input)
    const node = tree && findNodeAtLocation(tree, path)
    if (node && node.type !== "object") {
      const isPermissionKey = path[0] === "permission" && path.length === 2
      const replacement = isPermissionKey ? { "*": node.value, ...patch } : patch
      const edits = modify(input, path, replacement, {
        formattingOptions: { insertSpaces: true, tabSize: 2 },
      })
      return applyEdits(input, edits)
    }
  }
  // cypher_change end

  return Object.entries(patch).reduce((result, [key, value]) => patchJsonc(result, value, [...path, key]), input)
}

function writable(info: Info) {
  const { plugin_origins: _plugin_origins, ...next } = info
  return next
}

function writableGlobal(info: Info) {
  const next = writable(info)
  // When a user changes config from a value back to default in the Desktop app, we don't want to leave a blank `"shell": "",` key
  if ("shell" in next && next.shell === "") return { ...next, shell: undefined }
  return next
}

export const ConfigDirectoryTypoError = NamedError.create("ConfigDirectoryTypoError", {
  path: Schema.String,
  dir: Schema.String,
  suggestion: Schema.String,
})

export const layer = Layer.effect(
  Service,
  Effect.gen(function* () {
    const fs = yield* AppFileSystem.Service
    const authSvc = yield* Auth.Service
    const accountSvc = yield* Account.Service
    const env = yield* Env.Service
    const npmSvc = yield* Npm.Service
    const http = yield* HttpClient.HttpClient
    const git = yield* Git.Service // cypher_change

    const readConfigFile = (filepath: string) => fs.readFileStringSafe(filepath).pipe(Effect.orDie)

    const fetchRemoteJson = Effect.fnUntraced(function* <S extends Schema.Top>(
      url: string,
      headers: Record<string, string> | undefined,
      schema: S,
    ) {
      const response = yield* HttpClient.filterStatusOk(withTransientReadRetry(http))
        .execute(
          HttpClientRequest.get(url).pipe(HttpClientRequest.acceptJson, HttpClientRequest.setHeaders(headers ?? {})),
        )
        .pipe(
          Effect.catch((error) => Effect.die(new Error(`failed to fetch remote config from ${url}: ${String(error)}`))),
        )
      return yield* HttpClientResponse.schemaBodyJson(schema)(response).pipe(
        Effect.catch((error) => Effect.die(new Error(`failed to decode remote config from ${url}: ${String(error)}`))),
      )
    })

    const loadConfig = Effect.fnUntraced(function* (
      text: string,
      options: { path: string } | { dir: string; source: string },
      env?: Record<string, string>,
      // cypher_change start - trusted allows {env:}; fileScope confines untrusted {file:} reads to a root
      trusted?: boolean,
      fileScope?: ConfigVariable.FileScope,
      // cypher_change end
    ) {
      const source = "path" in options ? options.path : options.source
      const expanded = yield* Effect.promise(() =>
        ConfigVariable.substitute(
          "path" in options
            ? { text, type: "path", path: options.path, env, trusted, fileScope } // cypher_change
            : { text, type: "virtual", ...options, env, trusted, fileScope }, // cypher_change
        ),
      )
      const parsed = ConfigParse.jsonc(expanded, source)
      const data = ConfigParse.schema(Info, normalizeLoadedConfig(parsed, source), source)
      if (!("path" in options)) return data

      yield* Effect.promise(() => resolveLoadedPlugins(data, options.path))
      if (!data.$schema) {
        // cypher_change start
        data.$schema = "https://app.cypher.ai/config.json"
        const updated = text.replace(/^\s*\{/, '{\n  "$schema": "https://app.cypher.ai/config.json",')
        // cypher_change end
        yield* fs.writeFileString(options.path, updated).pipe(Effect.catch(() => Effect.void))
      }
      return data
    })

    const loadFile = Effect.fnUntraced(function* (
      filepath: string,
      env?: Record<string, string>,
      trusted?: boolean, // cypher_change
      fileScope?: ConfigVariable.FileScope, // cypher_change
    ) {
      log.info("loading", { path: filepath })
      const text = yield* readConfigFile(filepath)
      if (!text) return {} as Info
      return yield* loadConfig(text, { path: filepath }, env, trusted, fileScope) // cypher_change
    })

    let globalStamp = "" // cypher_change

    const loadGlobal = Effect.fnUntraced(function* (env?: Record<string, string>) {
      // cypher_change start
      yield* Effect.promise(() => CyphercodeConfig.migrateBashPermission())
      globalStamp = yield* CyphercodeGlobalConfigStamp.read(fs, Global.Path.config)
      // cypher_change end
      let result: Info = {}
      // Seed the default global config with the schema for editor completion, but avoid writing when the user
      // explicitly routes config through env-provided paths or content.
      if (!Flag.CYPHER_CONFIG && !Flag.CYPHER_CONFIG_DIR && !Flag.CYPHER_CONFIG_CONTENT) {
        const file = globalConfigFile()
        if (!existsSync(file)) {
          yield* fs
            .writeWithDirs(file, JSON.stringify({ $schema: "https://app.cypher.ai/config.json" }, null, 2))
            .pipe(Effect.catch(() => Effect.void))
        }
      }
      // cypher_change - global config is user-owned and trusted to resolve {file:}/{env:} tokens
      result = mergeConfig(result, yield* loadFile(path.join(Global.Path.config, "config.json"), env, true))
      // cypher_change start
      result = mergeConfig(result, yield* loadFile(path.join(Global.Path.config, "cypher.json"), env, true))
      result = mergeConfig(result, yield* loadFile(path.join(Global.Path.config, "cypher.jsonc"), env, true))
      // cypher_change end
      result = mergeConfig(result, yield* loadFile(path.join(Global.Path.config, "opencode.json"), env, true)) // cypher_change
      result = mergeConfig(result, yield* loadFile(path.join(Global.Path.config, "opencode.jsonc"), env, true)) // cypher_change

      const legacy = path.join(Global.Path.config, "config")
      if (existsSync(legacy)) {
        yield* Effect.promise(() =>
          import(pathToFileURL(legacy).href, { with: { type: "toml" } })
            .then(async (mod) => {
              const { provider, model, ...rest } = mod.default
              if (provider && model) result.model = `${provider}/${model}`
              result["$schema"] = "https://app.cypher.ai/config.json" // cypher_change
              result = mergeConfig(result, rest)
              await fsNode.writeFile(path.join(Global.Path.config, "config.json"), JSON.stringify(result, null, 2))
              await fsNode.unlink(legacy)
            })
            .catch(() => {}),
        )
      }

      globalStamp = yield* CyphercodeGlobalConfigStamp.read(fs, Global.Path.config) // cypher_change
      return result
    })

    const [cachedGlobal, invalidateGlobal] = yield* Effect.cachedInvalidateWithTTL(
      loadGlobal().pipe(
        Effect.tapError((error) =>
          Effect.sync(() => log.error("failed to load global config, using defaults", { error: String(error) })),
        ),
        Effect.orElseSucceed((): Info => ({})),
      ),
      Duration.infinity,
    )

    // cypher_change start - detect global config edits made by other Cypher processes
    const refreshGlobal = Effect.fnUntraced(function* () {
      const stamp = yield* CyphercodeGlobalConfigStamp.read(fs, Global.Path.config)
      if (!globalStamp || stamp === globalStamp) return false
      globalStamp = stamp
      yield* invalidateGlobal
      return true
    })
    // cypher_change end

    const getGlobal = Effect.fn("Config.getGlobal")(function* () {
      yield* refreshGlobal() // cypher_change
      return yield* cachedGlobal
    })

    const ensureGitignore = Effect.fn("Config.ensureGitignore")(function* (dir: string) {
      const gitignore = path.join(dir, ".gitignore")
      const hasIgnore = yield* fs.existsSafe(gitignore)
      if (!hasIgnore) {
        yield* fs
          .writeFileString(
            gitignore,
            // cypher_change start - added pnpm-lock.yaml, yarn.lock, agent-manager.json (not in upstream)
            [
              "node_modules",
              "package.json",
              "package-lock.json",
              "pnpm-lock.yaml",
              "bun.lock",
              "yarn.lock",
              ".gitignore",
              "agent-manager.json",
            ].join("\n"),
            // cypher_change end
          )
          .pipe(
            Effect.catchIf(
              (e) => e.reason._tag === "PermissionDenied" || e.reason._tag === "NotFound", // cypher_change - also ignore NotFound (broken symlink/junction on Windows)
              () => Effect.void,
            ),
          )
      }
    })

    const loadInstanceState = Effect.fn("Config.loadInstanceState")(
      function* (ctx: InstanceContext) {
        // cypher_change start - warning accumulator and legacy Cypher config
        const warnings: Warning[] = []
        // Untrusted project config may only read files inside this root (worktree, or directory for non-git projects).
        const projectRoot = ctx.worktree === "/" ? ctx.directory : ctx.worktree
        const auth = yield* authSvc.all().pipe(Effect.orDie)

        let result: Info = {}
        const legacy = yield* Effect.promise(() =>
          CyphercodeConfig.loadLegacyConfigs({
            projectDir: ctx.directory,
            merge: mergeConfigConcatArrays,
          }),
        )
        result = mergeConfigConcatArrays(result, legacy.config)
        warnings.push(...legacy.warnings)

        const orgModes = yield* Effect.promise(() => CyphercodeConfig.loadOrganizationModes(auth))
        if (Object.keys(orgModes.agents).length > 0) {
          result = mergeConfigConcatArrays(result, { agent: orgModes.agents })
        }
        warnings.push(...orgModes.warnings)
        // cypher_change end

        const authEnv: Record<string, string> = {}
        const consoleManagedProviders = new Set<string>()
        let activeOrgName: string | undefined

        const pluginScopeForSource = Effect.fnUntraced(function* (source: string) {
          if (source.startsWith("http://") || source.startsWith("https://")) return "global"
          if (source === "CYPHER_CONFIG_CONTENT") return "local"
          if (containsPath(source, ctx)) return "local"
          return "global"
        })

        const mergePluginOrigins = Effect.fnUntraced(function* (
          source: string,
          // mergePluginOrigins receives raw Specs from one config source, before provenance for this merge step
          // is attached.
          list: ConfigPlugin.Spec[] | undefined,
          // Scope can be inferred from the source path, but some callers already know whether the config should
          // behave as global or local and can pass that explicitly.
          kind?: ConfigPlugin.Scope,
        ) {
          if (!list?.length) return
          const hit = kind ?? (yield* pluginScopeForSource(source))
          // Merge newly seen plugin origins with previously collected ones, then dedupe by plugin identity while
          // keeping the winning source/scope metadata for downstream installs, writes, and diagnostics.
          const plugins = ConfigPlugin.deduplicatePluginOrigins([
            ...(result.plugin_origins ?? []),
            ...list.map((spec) => ({ spec, source, scope: hit })),
          ])
          result.plugin = plugins.map((item) => item.spec)
          result.plugin_origins = plugins
        })

        // cypher_change start
        const merge = Effect.fnUntraced(function* (source: string, next: Info, kind?: ConfigPlugin.Scope) {
          const scope = kind ?? (yield* pluginScopeForSource(source))
          const scoped = CyphercodeConfig.scopeIndexing(SandboxConfig.scope(next, scope), scope)
          result = mergeConfigConcatArrays(result, scoped)
          return yield* mergePluginOrigins(source, scoped.plugin, scope)
        })
        // cypher_change end

        for (const [key, value] of Object.entries(auth)) {
          if (value.type === "wellknown") {
            const url = key.replace(/\/+$/, "")
            authEnv[value.key] = value.token
            const wellknownURL = `${url}/.well-known/opencode`
            // cypher_change start
            const source = wellknownURL
            yield* Effect.gen(function* () {
              log.debug("fetching remote config", { url: wellknownURL })
              const wellknown = yield* fetchRemoteJson(wellknownURL, undefined, WellKnownConfig)
              const remote = yield* Effect.promise(() =>
                substituteWellKnownRemoteConfig({
                  value: wellknown.remote_config,
                  dir: url,
                  source: wellknownURL,
                  env: authEnv,
                }),
              )
              const fetchedConfig = remote
                ? yield* Effect.gen(function* () {
                    log.debug("fetching remote config", { url: remote.url })
                    const data = yield* fetchRemoteJson(remote.url, remote.headers, Schema.Json)
                    if (isRecord(data) && isRecord(data.config)) return data.config
                    if (isRecord(data)) return data
                    return yield* Effect.die(
                      new Error(`failed to decode remote config from ${remote.url}: expected object`),
                    )
                  })
                : {}
              const remoteConfig = mergeConfig(isRecord(wellknown.config) ? wellknown.config : {}, fetchedConfig)
              if (!remoteConfig.$schema) remoteConfig.$schema = "https://app.cypher.ai/config.json"
              const next = yield* loadConfig(
                JSON.stringify(remoteConfig),
                {
                  dir: path.dirname(source),
                  source,
                },
                authEnv,
                true, // cypher_change - well-known org config is a trusted source
              )
              yield* merge(source, next, "global")
              log.debug("loaded remote config from well-known", { url })
            }).pipe(
              Effect.catch((err: unknown) => {
                caughtWarning(warnings, source, err)
                log.warn("skipped remote config due to error", { url, err })
                return Effect.void
              }),
              Effect.catchDefect((err: unknown) => {
                caughtWarning(warnings, source, err)
                log.warn("skipped remote config due to error", { url, err })
                return Effect.void
              }),
            )
            // cypher_change end
          }
        }

        // cypher_change start - capture global config failures as warnings
        const global = yield* (Object.keys(authEnv).length ? loadGlobal(authEnv) : getGlobal()).pipe(
          Effect.catchDefect((err: unknown) => {
            caughtWarning(warnings, "global config", err)
            return Effect.succeed({} as Info)
          }),
        )
        // cypher_change end

        yield* merge(Global.Path.config, global, "global")

        if (Flag.CYPHER_CONFIG) {
          // cypher_change start - capture CYPHER_CONFIG failures as warnings
          yield* merge(
            Flag.CYPHER_CONFIG,
            // cypher_change - CYPHER_CONFIG is an explicit user-provided path, trusted for {file:}/{env:}
            yield* loadFile(Flag.CYPHER_CONFIG, authEnv, true).pipe(
              Effect.catchDefect((err: unknown) => {
                caughtWarning(warnings, Flag.CYPHER_CONFIG!, err)
                return Effect.succeed({} as Info)
              }),
            ),
          )
          // cypher_change end
          log.debug("loaded custom config", { path: Flag.CYPHER_CONFIG })
        }

        if (!Flag.CYPHER_DISABLE_PROJECT_CONFIG) {
          // cypher_change start - also discover cypher.json project files
          for (const name of ["cypher", "opencode"] as const) {
            for (const file of yield* ConfigPaths.files(name, ctx.directory, ctx.worktree).pipe(Effect.orDie)) {
              yield* merge(
                file,
                // cypher_change - project config is untrusted: {env:} rejected, {file:} confined to projectRoot
                yield* loadFile(file, authEnv, false, { root: projectRoot, source: file }).pipe(
                  Effect.catchDefect((err: unknown) => {
                    caughtWarning(warnings, file, err)
                    return Effect.succeed({} as Info)
                  }),
                ),
                "local",
              )
            }
          }
          // cypher_change end
        }

        result.agent = result.agent || {}
        result.mode = result.mode || {}
        result.plugin = result.plugin || []

        // cypher_change start - include config directories from the primary checkout
        const directories = yield* ConfigPaths.directories(ctx.directory, ctx.worktree)
        const primary = Flag.CYPHER_DISABLE_PROJECT_CONFIG
          ? []
          : yield* primaryPaths(ctx.directory, ctx.worktree, [".cypher", ".cypher"])
        // Load primary fallbacks before active-worktree config, then track them as local.
        directories.splice(1, 0, ...primary)
        const primarySet = new Set(primary)
        // cypher_change end

        if (Flag.CYPHER_CONFIG_DIR) {
          log.debug("loading config from CYPHER_CONFIG_DIR", { path: Flag.CYPHER_CONFIG_DIR })
        }

        const deps: Fiber.Fiber<void>[] = []

        // cypher_change start
        for (const dir of unique(directories)) {
          const scope = primarySet.has(dir) ? "local" : undefined
          // cypher_change - trust {file:}/{env:} only for global-scoped config dirs, never project ones
          const dirScope = scope ?? (yield* pluginScopeForSource(dir))
          const dirTrusted = dirScope === "global"
          // cypher_change - untrusted config dirs confine {file:} reads to projectRoot
          const dirFileScope = dirTrusted ? undefined : { root: projectRoot, source: dir }
          if (CyphercodeConfig.isConfigDir(dir, Flag.CYPHER_CONFIG_DIR)) {
            for (const file of CyphercodeConfig.ALL_CONFIG_FILES) {
              const source = path.join(dir, file)
              log.debug(`loading config from ${source}`)
              // cypher_change - untrusted config dirs confine {file:} reads to projectRoot
              const fileScope = dirTrusted ? undefined : { root: projectRoot, source }
              yield* merge(
                source,
                yield* loadFile(source, authEnv, dirTrusted, fileScope).pipe(
                  // cypher_change
                  Effect.catchDefect((err: unknown) => {
                    caughtWarning(warnings, source, err)
                    return Effect.succeed({} as Info)
                  }),
                ),
                dirScope, // cypher_change
              )
              result.agent ??= {}
              result.mode ??= {}
              result.plugin ??= []
            }
          }
          // cypher_change end

          yield* ensureGitignore(dir).pipe(Effect.orDie)

          const dep = yield* npmSvc
            .install(dir, {
              add: [
                {
                  name: "@cypher/plugin",
                  version: InstallationLocal ? undefined : InstallationVersion,
                },
              ],
            })
            .pipe(
              Effect.exit,
              Effect.tap((exit) =>
                Exit.isFailure(exit)
                  ? Effect.sync(() => {
                      log.warn("background dependency install failed", { dir, error: String(exit.cause) })
                    })
                  : Effect.void,
              ),
              Effect.asVoid,
              Effect.forkDetach,
            )
          deps.push(dep)

          // cypher_change start - propagate parse errors to the Warning accumulator
          result.command = mergeDeep(
            result.command ?? {},
            yield* Effect.promise(() => ConfigCommand.load(dir, warnings)),
          )
          result.agent = mergeDeep(
            result.agent ?? {},
            yield* Effect.promise(() => ConfigAgent.load(dir, warnings, dirTrusted, dirFileScope)), // cypher_change
          )
          result.agent = mergeDeep(result.agent ?? {}, yield* Effect.promise(() => ConfigAgent.loadMode(dir, warnings)))
          // cypher_change end
          // cypher_change - Auto-discovered plugins under config directories are already local files, so ConfigPlugin.load
          // returns normalized Specs and we only need to attach origin metadata here.
          const list = yield* Effect.promise(() => ConfigPlugin.load(dir))
          yield* mergePluginOrigins(dir, list, dirScope) // cypher_change
        }

        if (process.env.CYPHER_CONFIG_CONTENT) {
          // cypher_change start - capture CYPHER_CONFIG_CONTENT parse failures as warnings
          const source = "CYPHER_CONFIG_CONTENT"
          yield* merge(
            source,
            yield* loadConfig(
              process.env.CYPHER_CONFIG_CONTENT,
              {
                dir: ctx.directory,
                source,
              },
              undefined,
              true, // cypher_change - CYPHER_CONFIG_CONTENT is user-provided, trusted for {file:}/{env:}
            ).pipe(
              Effect.tap(() => Effect.sync(() => log.debug("loaded custom config from CYPHER_CONFIG_CONTENT"))),
              Effect.catchDefect((err: unknown) => {
                caughtWarning(warnings, source, err)
                return Effect.succeed({} as Info)
              }),
            ),
            "local",
          )
          // cypher_change end
        }

        const activeAccount = Option.getOrUndefined(
          yield* accountSvc.active().pipe(Effect.catch(() => Effect.succeed(Option.none()))),
        )
        if (activeAccount?.active_org_id) {
          const accountID = activeAccount.id
          const orgID = activeAccount.active_org_id
          const url = activeAccount.url
          yield* Effect.gen(function* () {
            const [configOpt, tokenOpt] = yield* Effect.all(
              [accountSvc.config(accountID, orgID), accountSvc.token(accountID)],
              { concurrency: 2 },
            )
            if (Option.isSome(tokenOpt)) {
              process.env["CYPHER_CONSOLE_TOKEN"] = tokenOpt.value
              yield* env.set("CYPHER_CONSOLE_TOKEN", tokenOpt.value)
            }

            if (Option.isSome(configOpt)) {
              const source = `${url}/api/config`
              const next = yield* loadConfig(
                JSON.stringify(configOpt.value),
                {
                  dir: path.dirname(source),
                  source,
                },
                undefined,
                true, // cypher_change - console-managed org config is a trusted source
              )
              for (const providerID of Object.keys(next.provider ?? {})) {
                consoleManagedProviders.add(providerID)
              }
              yield* merge(source, next, "global")
            }
          }).pipe(
            Effect.withSpan("Config.loadActiveOrgConfig"),
            Effect.catch((err) => {
              log.debug("failed to fetch remote account config", {
                error: err instanceof Error ? err.message : String(err),
              })
              return Effect.void
            }),
          )
        }

        const managedDir = ConfigManaged.managedConfigDir()
        // cypher_change start - include cypher.json/cypher.jsonc in managed dir loading
        if (existsSync(managedDir)) {
          for (const file of CyphercodeConfig.ALL_CONFIG_FILES) {
            const source = path.join(managedDir, file)
            // cypher_change - MDM/enterprise-managed config is a trusted source
            yield* merge(source, yield* loadFile(source, undefined, true), "global")
          }
        }
        // cypher_change end

        // macOS managed preferences (.mobileconfig deployed via MDM) override everything
        // cypher_change start
        const managed = yield* Effect.promise(() => ConfigManaged.readManagedPreferences())
        if (managed) {
          yield* merge(
            managed.source,
            yield* loadConfig(
              managed.text,
              {
                dir: path.dirname(managed.source),
                source: managed.source,
              },
              undefined,
              true, // cypher_change - MDM-managed preferences are a trusted source
            ),
            "global",
          )
        }
        // cypher_change end

        for (const [name, mode] of Object.entries(result.mode ?? {})) {
          result.agent = mergeDeep(result.agent ?? {}, {
            [name]: {
              ...mode,
              mode: "primary" as const,
            },
          })
        }

        if (Flag.CYPHER_PERMISSION) {
          try {
            result.permission = mergeDeep(result.permission ?? {}, JSON.parse(Flag.CYPHER_PERMISSION))
          } catch (err) {
            log.warn("CYPHER_PERMISSION contains invalid JSON, skipping", { err })
          }
        }

        if (result.tools) {
          const perms: Record<string, ConfigPermission.Action> = {}
          for (const [tool, enabled] of Object.entries(result.tools)) {
            const action: ConfigPermission.Action = enabled ? "allow" : "deny"
            if (tool === "write" || tool === "edit" || tool === "patch") {
              perms.edit = action
              continue
            }
            perms[tool] = action
          }
          result.permission = mergeDeep(perms, result.permission ?? {})
        }

        if (!result.username) {
          try {
            result.username = os.userInfo().username || "user"
          } catch (err) {
            log.warn("failed to read system username, using fallback", { err })
            result.username = "user"
          }
        }

        if (result.autoshare === true && !result.share) {
          result.share = "auto"
        }

        if (Flag.CYPHER_DISABLE_AUTOCOMPACT) {
          result.compaction = { ...result.compaction, auto: false }
        }
        if (Flag.CYPHER_DISABLE_PRUNE) {
          result.compaction = { ...result.compaction, prune: false }
        }
        // cypher_change start — inject Cypher default plugins into both plugin list and origins
        CyphercodeDefaultPlugins.apply(result, { disabled: Flag.CYPHER_DISABLE_DEFAULT_PLUGINS, log })
        // cypher_change end

        return {
          config: result,
          directories,
          deps,
          warnings, // cypher_change
          consoleState: {
            consoleManagedProviders: Array.from(consoleManagedProviders),
            activeOrgName,
            switchableOrgCount: 0,
          },
        }
      },
      Effect.provideService(AppFileSystem.Service, fs),
    )

    const state = yield* InstanceState.make<State>(
      Effect.fn("Config.state")(function* (ctx) {
        return yield* loadInstanceState(ctx).pipe(Effect.provideService(Git.Service, git), Effect.orDie) // cypher_change
      }),
    )

    const get = Effect.fn("Config.get")(function* () {
      // cypher_change start - reload instance config when global config changed elsewhere
      if (yield* refreshGlobal()) {
        yield* InstanceState.invalidate(state).pipe(Effect.catchCause(() => Effect.void))
      }
      // cypher_change end
      return yield* InstanceState.use(state, (s) => s.config)
    })

    const directories = Effect.fn("Config.directories")(function* () {
      return yield* InstanceState.use(state, (s) => s.directories)
    })

    const getConsoleState = Effect.fn("Config.getConsoleState")(function* () {
      return yield* InstanceState.use(state, (s) => s.consoleState)
    })

    const waitForDependencies = Effect.fn("Config.waitForDependencies")(function* () {
      yield* InstanceState.useEffect(state, (s) =>
        Effect.forEach(s.deps, Fiber.join, { concurrency: "unbounded" }).pipe(Effect.asVoid),
      )
    })

    const update = Effect.fn("Config.update")(function* (config: Info) {
      // cypher_change start - delegate Cypher project config update behavior.
      const ctx = yield* InstanceState.context
      yield* CyphercodeConfig.updateProjectConfig({
        fs,
        directory: ctx.directory,
        worktree: ctx.worktree,
        config,
        read: readConfigFile,
        parse: (input, file) => ConfigParse.schema(Info, ConfigParse.jsonc(input, file), file),
        patch: (input, patch) => patchJsonc(input, patch),
        writable,
      })
      yield* InstanceState.invalidate(state)
      yield* Effect.sync(() =>
        GlobalBus.emit("event", {
          directory: ctx.directory,
          payload: {
            type: Event.ConfigUpdated.type,
            properties: {},
          },
        }),
      )
    })

    const warnings = Effect.fn("Config.warnings")(function* () {
      return yield* InstanceState.use(state, (s) => s.warnings)
    })
    // cypher_change end

    const invalidate = Effect.fn("Config.invalidate")(function* () {
      yield* invalidateGlobal
    })

    // cypher_change start - add dispose option to skip Instance.disposeAll for permission-only changes
    const updateGlobal = Effect.fn("Config.updateGlobal")(function* (config: Info, options?: { dispose?: boolean }) {
      const dispose = options?.dispose ?? true
      // cypher_change end
      const file = globalConfigFile()
      const before = (yield* readConfigFile(file)) ?? "{}"
      const patch = writableGlobal(config)

      let next: Info
      let changed: boolean
      if (!file.endsWith(".jsonc")) {
        const existing = ConfigParse.schema(Info, ConfigParse.jsonc(before, file), file)
        const merged = CyphercodeConfig.mergeConfig(writable(existing), patch) // cypher_change
        const serialized = JSON.stringify(merged, null, 2)
        changed = serialized !== before
        if (changed) yield* fs.writeFileString(file, serialized).pipe(Effect.orDie)
        next = merged
      } else {
        const updated = patchJsonc(before, patch)
        next = ConfigParse.schema(Info, ConfigParse.jsonc(updated, file), file)
        changed = updated !== before
        if (changed) yield* fs.writeFileString(file, updated).pipe(Effect.orDie)
      }

      // cypher_change start - skip dispose when caller opts out
      if (!dispose) {
        yield* invalidateGlobal
        yield* InstanceState.invalidate(state).pipe(Effect.catchCause(() => Effect.void))
        yield* Effect.sync(() =>
          GlobalBus.emit("event", {
            directory: "global",
            payload: {
              type: Event.ConfigUpdated.type,
              properties: {},
            },
          }),
        ).pipe(Effect.catchCause(() => Effect.void))
        return { info: next, changed }
      }
      // cypher_change end

      if (changed) yield* invalidate()
      // cypher_change start - hot-reload global config changes in the active instance
      if (changed) {
        yield* InstanceState.invalidate(state).pipe(Effect.catchCause(() => Effect.void))
        yield* Effect.sync(() =>
          GlobalBus.emit("event", {
            directory: "global",
            payload: {
              type: Event.ConfigUpdated.type,
              properties: {},
            },
          }),
        ).pipe(Effect.catchCause(() => Effect.void))
      }
      // cypher_change end
      return { info: next, changed }
    })

    return Service.of({
      get,
      getGlobal,
      getConsoleState,
      update,
      updateGlobal,
      invalidate,
      directories,
      waitForDependencies,
      warnings, // cypher_change
    })
  }),
)

export const defaultLayer = layer.pipe(
  Layer.provide(Git.defaultLayer), // cypher_change
  Layer.provide(EffectFlock.defaultLayer),
  Layer.provide(AppFileSystem.defaultLayer),
  Layer.provide(Env.defaultLayer),
  Layer.provide(Auth.defaultLayer),
  Layer.provide(Account.defaultLayer),
  Layer.provide(Npm.defaultLayer),
  Layer.provide(FetchHttpClient.layer),
)

export * as Config from "./config"
