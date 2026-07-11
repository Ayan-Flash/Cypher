// ============================================================================
// Plugin
// ============================================================================
export { CypherAuthPlugin, default } from "./plugin.js"

// ============================================================================
// Provider
// ============================================================================
export { createCypher } from "./provider.js"
export { createCypherDebug } from "./provider-debug.js"
export { cypherCustomLoader } from "./loader.js"
export { buildCypherHeaders, getEditorNameHeader, getFeatureHeader, getDefaultHeaders, getUserAgent } from "./headers.js"

// ============================================================================
// Auth
// ============================================================================
export { authenticateWithDeviceAuth } from "./auth/device-auth.js"
export { authenticateWithDeviceAuthTUI } from "./auth/device-auth-tui.js"
export { getCypherUrlFromToken, isValidCyphercodeToken, getApiKey } from "./auth/token.js"
export { poll, formatTimeRemaining } from "./auth/polling.js"
export { migrateLegacyCypherAuth, LEGACY_CONFIG_PATH } from "./auth/legacy-migration.js"

// ============================================================================
// API
// ============================================================================
export {
  fetchProfile,
  fetchBalance,
  fetchProfileWithBalance,
  fetchDefaultModel,
  getCypherProfile,
  defaultOrganizationId,
  getCypherBalance,
  getCypherDefaultModel,
  promptOrganizationSelection,
} from "./api/profile.js"
export { fetchCypherPassState } from "./api/cypher-pass.js"
export {
  fetchCypherModels,
  type CypherModelsResult,
  fetchCypherImageModels,
  type CypherImageModel,
  type CypherImageModelsResult,
} from "./api/models.js"
export {
  EMPTY_CYPHER_EMBEDDING_MODEL_CATALOG,
  fetchCypherEmbeddingModelCatalog,
  type CypherEmbeddingModel,
  type CypherEmbeddingModelCatalog,
  type CypherEmbeddingModelCatalogIssue,
} from "./api/embedding-models.js"
export { resolveCypherGatewayBaseUrl, resolveCypherOpenRouterBaseUrl } from "./api/url.js"
export {
  AUTOCOMPLETE_MODELS,
  DEFAULT_AUTOCOMPLETE_MODEL,
  getAutocompleteModel,
  getAutocompleteModelById,
  validAutocompleteModel,
  validAutocompleteProvider,
  type AutocompleteModelDef,
  type AutocompleteProviderID,
} from "./autocomplete.js"
export {
  fetchOrganizationModes,
  clearModesCache,
  type OrganizationMode,
  type OrganizationModeConfig,
} from "./api/modes.js"
export { fetchCyphercodeNotifications, type CyphercodeNotification } from "./api/notifications.js"
export { fetchCloudSession, fetchCloudSessionForImport, importSessionToDb } from "./cloud-sessions.js"

// ============================================================================
// Server Routes (optional - requires hono and OpenCode dependencies)
// ============================================================================
export { createCypherRoutes } from "./server/routes.js"
export {
  GatewayError,
  UnauthorizedError,
  getOrganizationId,
  getClawChatCredentials,
  getClawStatus,
  getCloudSessions,
  getNotifications,
  getProfile,
  getToken,
  normalizeClawStatus,
  setOrganization,
} from "./server/handlers.js"

// ============================================================================
// Note: TUI exports moved to separate entry point
// ============================================================================
// For TUI components and commands, import from "@cypher/cypher-gateway/tui"
// This avoids circular dependencies with opencode TUI infrastructure

// ============================================================================
// Types
// ============================================================================
export type {
  // Auth types
  DeviceAuthInitiateResponse,
  DeviceAuthPollResponse,
  Organization,
  CyphercodeProfile,
  CyphercodeBalance,
  CypherPassState,
  PollOptions,
  PollResult,
  // Provider types
  CypherProvider,
  CypherProviderOptions,
  CypherMetadata,
  CustomLoaderResult,
  ProviderInfo,
  LanguageModelV3,
} from "./types.js"

// ============================================================================
// Constants
// ============================================================================
export {
  ENV_CYPHER_API_URL,
  DEFAULT_CYPHER_API_URL,
  CYPHER_API_BASE,
  CYPHER_CHAT_URL,
  CYPHER_EVENT_SERVICE_URL,
  CYPHER_OPENROUTER_BASE,
  POLL_INTERVAL_MS,
  DEFAULT_MODEL,
  DEFAULT_FREE_MODEL,
  TOKEN_EXPIRATION_MS,
  USER_AGENT_BASE,
  CONTENT_TYPE,
  DEFAULT_PROVIDER_NAME,
  ANONYMOUS_API_KEY,
  MODELS_FETCH_TIMEOUT_MS,
  HEADER_ORGANIZATIONID,
  HEADER_TASKID,
  HEADER_PARENT_TASKID,
  HEADER_PROJECTID,
  HEADER_TESTER,
  HEADER_EDITORNAME,
  HEADER_MACHINEID,
  HEADER_FEATURE,
  DEFAULT_EDITOR_NAME,
  ENV_EDITOR_NAME,
  ENV_VERSION,
  TESTER_SUPPRESS_VALUE,
  ENV_FEATURE,
  PROMPTS,
  AI_SDK_PROVIDERS,
} from "./api/constants.js"
