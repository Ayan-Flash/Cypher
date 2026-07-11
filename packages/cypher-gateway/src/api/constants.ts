/**
 * Cypher Gateway Configuration Constants
 * Centralized configuration for all API endpoints, headers, and settings
 */

/** Environment variable for custom Cypher API URL */
export const ENV_CYPHER_API_URL = "CYPHER_API_URL"

/** Default Cypher API URL */
export const DEFAULT_CYPHER_API_URL = "https://api.cypher.ai"

/** Base URL for Cypher API - can be overridden by CYPHER_API_URL env var */
export const CYPHER_API_BASE = process.env[ENV_CYPHER_API_URL] || DEFAULT_CYPHER_API_URL

/** Environment variable for custom Cypher Chat URL */
export const CYPHER_CHAT_URL_ENV = "CYPHER_CHAT_URL"

/** Default Cypher Chat URL (REST endpoint for messages, conversations, etc.) */
export const CYPHER_DEFAULT_CHAT_URL = "https://chat.cypherapps.io"

/** Base URL for Cypher Chat - can be overridden by CYPHER_CHAT_URL env var */
export const CYPHER_CHAT_URL = process.env[CYPHER_CHAT_URL_ENV] || CYPHER_DEFAULT_CHAT_URL

/** Environment variable for custom Event Service URL */
export const CYPHER_EVENT_SERVICE_URL_ENV = "EVENT_SERVICE_URL"

/** Default Event Service URL (WebSocket endpoint for cypher-chat events) */
export const CYPHER_DEFAULT_EVENT_SERVICE_URL = "wss://events.cypherapps.io"

/** Base URL for Event Service - can be overridden by EVENT_SERVICE_URL env var */
export const CYPHER_EVENT_SERVICE_URL = process.env[CYPHER_EVENT_SERVICE_URL_ENV] || CYPHER_DEFAULT_EVENT_SERVICE_URL

/** Default base URL for OpenRouter-compatible endpoint */
export const CYPHER_OPENROUTER_BASE = `${CYPHER_API_BASE}/api/openrouter`

/** Device auth polling interval in milliseconds */
export const POLL_INTERVAL_MS = 3000

/** Default model for authenticated users */
export const DEFAULT_MODEL = "cypher-auto/free"

/** Default model for anonymous/free usage */
export const DEFAULT_FREE_MODEL = "cypher-auto/free"

/** Token expiration duration in milliseconds (1 year) */
export const TOKEN_EXPIRATION_MS = 365 * 24 * 60 * 60 * 1000

/** User-Agent header base value for requests */
export const USER_AGENT_BASE = "opencode-cypher-provider"

/** Content-Type header value for requests */
export const CONTENT_TYPE = "application/json"

/** Default provider name */
export const DEFAULT_PROVIDER_NAME = "cypher"

/** Default API key for anonymous requests */
export const ANONYMOUS_API_KEY = "anonymous"

/** Fetch timeout for model requests in milliseconds (10 seconds) */
export const MODELS_FETCH_TIMEOUT_MS = 10 * 1000

/**
 * Header constants for Cypher API requests
 */
export const HEADER_ORGANIZATIONID = "X-CYPHER-ORGANIZATIONID"
export const HEADER_TASKID = "X-CYPHER-TASKID"
export const HEADER_PARENT_TASKID = "X-CYPHER-PARENT-TASKID"
export const HEADER_PROJECTID = "X-CYPHER-PROJECTID"
export const HEADER_TESTER = "X-CYPHER-TESTER"
export const HEADER_EDITORNAME = "X-CYPHER-EDITORNAME"
export const HEADER_MACHINEID = "X-CYPHER-MACHINEID"

/** Default editor name value */
export const DEFAULT_EDITOR_NAME = "Cypher CLI"

/** Environment variable name for custom editor name */
export const ENV_EDITOR_NAME = "CYPHER_EDITOR_NAME"

/** Environment variable name for version (set by CLI at startup) */
export const ENV_VERSION = "CYPHER_VERSION"

/** Tester header value for suppressing warnings */
export const TESTER_SUPPRESS_VALUE = "SUPPRESS"

/** Header name for feature tracking */
export const HEADER_FEATURE = "X-CYPHER-FEATURE"

/** Environment variable name for feature override */
export const ENV_FEATURE = "CYPHER_FEATURE"

export const PROMPTS = [
  "codex",
  "gemini",
  "beast",
  "anthropic",
  "trinity",
  "anthropic_without_todo",
  "ling",
  "gpt55",
] as const

export const AI_SDK_PROVIDERS = [
  "alibaba",
  "anthropic",
  "mistral",
  "openai",
  "openai-compatible",
  "openrouter",
] as const
