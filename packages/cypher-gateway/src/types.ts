import type { LanguageModel, Provider, Provider as SDK } from "ai"
import type { LanguageModelV3 } from "@openrouter/ai-sdk-provider"

// ============================================================================
// Authentication Types
// ============================================================================

export interface DeviceAuthInitiateResponse {
  code: string
  verificationUrl: string
  expiresIn: number
}

export interface DeviceAuthPollResponse {
  status: "pending" | "approved" | "denied" | "expired"
  token?: string
  userEmail?: string
}

export interface Organization {
  id: string
  name: string
  role: string
}

export interface CyphercodeProfile {
  email: string
  name?: string
  organizations?: Organization[]
  selectedOrganizationId?: string
  hasPersonalAccount?: boolean
}

export interface CyphercodeBalance {
  balance: number
}

export interface CypherPassState {
  currentPeriodBaseCreditsUsd: number
  currentPeriodUsageUsd: number
  currentPeriodBonusCreditsUsd: number
  nextBillingAt?: string | null
}

export interface PollOptions<T> {
  interval: number
  maxAttempts: number
  pollFn: () => Promise<PollResult<T>>
}

export interface PollResult<T> {
  continue: boolean
  data?: T
  error?: Error
}

// ============================================================================
// Provider Types
// ============================================================================

/**
 * Options for creating a Cypher provider instance
 */
export interface CypherProviderOptions {
  /**
   * Cypher authentication token
   */
  cypherToken?: string

  /**
   * Organization ID for multi-tenant setups
   */
  cypherOrganizationId?: string

  /**
   * Model ID to use (e.g., "anthropic/claude-sonnet-4")
   */
  cypherModel?: string

  /**
   * Specific OpenRouter provider to use
   */
  openRouterSpecificProvider?: string

  /**
   * Base URL for the Cypher API
   * Can be overridden by CYPHER_API_URL environment variable
   * @default "https://api.cypher.ai"
   */
  baseURL?: string

  /**
   * Custom headers to include in requests
   */
  headers?: Record<string, string>

  /**
   * API key (alternative to cypherToken)
   */
  apiKey?: string

  /**
   * Provider name for identification
   */
  name?: string

  /**
   * Data collection preference for upstream provider routing
   */
  dataCollection?: "allow" | "deny"

  /**
   * Custom fetch function
   */
  fetch?: typeof fetch

  /**
   * Request timeout in milliseconds
   */
  timeout?: number | false
}

/**
 * Metadata for API requests
 */
export interface CypherMetadata {
  /**
   * Task ID for tracking
   */
  taskId?: string

  /**
   * Project ID for organization tracking
   */
  projectId?: string

  /**
   * Mode of operation (e.g., "code", "chat")
   */
  mode?: string
}

/**
 * Custom loader return type
 */
export interface CustomLoaderResult {
  /**
   * Whether to automatically load this provider
   */
  autoload: boolean

  /**
   * Custom function to get a model instance
   */
  getModel?: (sdk: SDK, modelID: string, options?: Record<string, any>) => Promise<LanguageModelV3>

  /**
   * Options to merge with provider configuration
   */
  options?: Record<string, any>
}

/**
 * Provider info type (minimal definition needed for loader)
 */
export interface ProviderInfo {
  id: string
  name: string
  source: "env" | "config" | "custom" | "api"
  env: string[]
  key?: string
  options: Record<string, any>
  models: Record<string, any>
}

export type CypherProvider = Provider & {
  alibaba(modelId: string): LanguageModel
  anthropic(modelId: string): LanguageModel
  mistral(modelId: string): LanguageModel
  openai(modelId: string): LanguageModel
  openaiCompatible(modelId: string): LanguageModel
}

// Re-export LanguageModelV3 for convenience
export type { LanguageModelV3 }
