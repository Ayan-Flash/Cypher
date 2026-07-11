import { Config, ConfigProvider, Context, Effect, Layer, Option } from "effect"
import { ConfigService } from "@/effect/config-service"

const bool = (name: string) => Config.boolean(name).pipe(Config.withDefault(false))
const positiveInteger = (name: string) =>
  Config.number(name).pipe(
    Config.map((value) => (Number.isInteger(value) && value > 0 ? value : undefined)),
    Config.orElse(() => Config.succeed(undefined)),
  )
const experimental = bool("CYPHER_EXPERIMENTAL")
const enabledByExperimental = (name: string) =>
  Config.all({ experimental, enabled: Config.boolean(name).pipe(Config.option) }).pipe(
    Config.map((flags) => Option.getOrElse(flags.enabled, () => flags.experimental)),
  )

export class Service extends ConfigService.Service<Service>()("@opencode/RuntimeFlags", {
  autoShare: bool("CYPHER_AUTO_SHARE"),
  pure: bool("CYPHER_PURE"),
  disableDefaultPlugins: bool("CYPHER_DISABLE_DEFAULT_PLUGINS"),
  disableChannelDb: bool("CYPHER_DISABLE_CHANNEL_DB"),
  disableEmbeddedWebUi: bool("CYPHER_DISABLE_EMBEDDED_WEB_UI"),
  disableExternalSkills: bool("CYPHER_DISABLE_EXTERNAL_SKILLS"),
  disableLspDownload: bool("CYPHER_DISABLE_LSP_DOWNLOAD"),
  skipMigrations: bool("CYPHER_SKIP_MIGRATIONS"),
  disableClaudeCodePrompt: Config.all({
    broad: bool("CYPHER_DISABLE_CLAUDE_CODE"),
    direct: bool("CYPHER_DISABLE_CLAUDE_CODE_PROMPT"),
  }).pipe(Config.map((flags) => flags.broad || flags.direct)),
  disableClaudeCodeSkills: Config.all({
    broad: bool("CYPHER_DISABLE_CLAUDE_CODE"),
    direct: bool("CYPHER_DISABLE_CLAUDE_CODE_SKILLS"),
  }).pipe(Config.map((flags) => flags.broad || flags.direct)),
  enableExa: Config.all({
    experimental,
    enabled: bool("CYPHER_ENABLE_EXA"),
    legacy: bool("CYPHER_EXPERIMENTAL_EXA"),
  }).pipe(Config.map((flags) => flags.experimental || flags.enabled || flags.legacy)),
  enableParallel: Config.all({
    enabled: bool("CYPHER_ENABLE_PARALLEL"),
    legacy: bool("CYPHER_EXPERIMENTAL_PARALLEL"),
  }).pipe(Config.map((flags) => flags.enabled || flags.legacy)),
  enableExperimentalModels: bool("CYPHER_ENABLE_EXPERIMENTAL_MODELS"),
  enableQuestionTool: bool("CYPHER_ENABLE_QUESTION_TOOL"),
  experimentalScout: enabledByExperimental("CYPHER_EXPERIMENTAL_SCOUT"),
  experimentalBackgroundSubagents: enabledByExperimental("CYPHER_EXPERIMENTAL_BACKGROUND_SUBAGENTS"),
  experimentalLspTy: bool("CYPHER_EXPERIMENTAL_LSP_TY"),
  experimentalLspTool: enabledByExperimental("CYPHER_EXPERIMENTAL_LSP_TOOL"),
  experimentalOxfmt: enabledByExperimental("CYPHER_EXPERIMENTAL_OXFMT"),
  experimentalPlanMode: enabledByExperimental("CYPHER_EXPERIMENTAL_PLAN_MODE"),
  experimentalEventSystem: enabledByExperimental("CYPHER_EXPERIMENTAL_EVENT_SYSTEM"),
  experimentalWorkspaces: enabledByExperimental("CYPHER_EXPERIMENTAL_WORKSPACES"),
  experimentalIconDiscovery: enabledByExperimental("CYPHER_EXPERIMENTAL_ICON_DISCOVERY"),
  outputTokenMax: positiveInteger("CYPHER_EXPERIMENTAL_OUTPUT_TOKEN_MAX"),
  bashDefaultTimeoutMs: positiveInteger("CYPHER_EXPERIMENTAL_BASH_DEFAULT_TIMEOUT_MS"),
  experimentalNativeLlm: bool("CYPHER_EXPERIMENTAL_NATIVE_LLM"),
  experimentalWebSockets: bool("CYPHER_EXPERIMENTAL_WEBSOCKETS"),
  client: Config.string("CYPHER_CLIENT").pipe(Config.withDefault("cli")),
}) {}

export type Info = Context.Service.Shape<typeof Service>

const emptyConfigLayer = Service.defaultLayer.pipe(
  Layer.provide(ConfigProvider.layer(ConfigProvider.fromUnknown({}))),
  Layer.orDie,
)

export const layer = (overrides: Partial<Info> = {}) =>
  Layer.effect(
    Service,
    Effect.gen(function* () {
      const flags = yield* Service
      return Service.of({ ...flags, ...overrides })
    }),
  ).pipe(Layer.provide(emptyConfigLayer))

export const defaultLayer = Service.defaultLayer.pipe(Layer.orDie)

export * as RuntimeFlags from "./runtime-flags"
