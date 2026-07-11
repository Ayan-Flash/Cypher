import type { NamedError } from "@opencode-ai/core/util/error"
import { isRecord } from "@/util/record"

export const CYPHER_ERROR_CODES = {
  PAID_MODEL_AUTH_REQUIRED: "PAID_MODEL_AUTH_REQUIRED",
  PROMOTION_MODEL_LIMIT_REACHED: "PROMOTION_MODEL_LIMIT_REACHED",
} as const

export type CypherErrorCode = (typeof CYPHER_ERROR_CODES)[keyof typeof CYPHER_ERROR_CODES]

const CYPHER_ERROR_CODE_VALUES = Object.values(CYPHER_ERROR_CODES) as string[]

/**
 * Check if an error is a Cypher-specific error (has a known Cypher error code in responseBody).
 * Currently all Cypher errors are non-retryable, but this may change in the future.
 */
export function isCypherError(error: ReturnType<NamedError["toObject"]>): boolean {
  return parseCypherErrorCode(error) !== undefined
}

/**
 * Get a user-friendly title for a Cypher error code.
 */
export function cypherErrorTitle(code: CypherErrorCode): string {
  switch (code) {
    case CYPHER_ERROR_CODES.PAID_MODEL_AUTH_REQUIRED:
      return "You need to sign in to use this model"
    case CYPHER_ERROR_CODES.PROMOTION_MODEL_LIMIT_REACHED:
      return "You need to sign up to keep going"
  }
}

/**
 * Get a user-friendly description for a Cypher error code.
 */
export function cypherErrorDescription(code: CypherErrorCode): string {
  switch (code) {
    case CYPHER_ERROR_CODES.PAID_MODEL_AUTH_REQUIRED:
      return "Sign in or create an account to access over 500 models, use credits at cost, or bring your own key."
    case CYPHER_ERROR_CODES.PROMOTION_MODEL_LIMIT_REACHED:
      return "Sign up for free to continue and explore 500 other models. Takes 2 minutes, no credit card required. Or come back later."
  }
}

/**
 * Show a warning toast with the appropriate Cypher error title/description.
 * Caller should check isCypherError() first.
 */
export function showCypherErrorToast(
  error: ReturnType<NamedError["toObject"]>,
  toast: { show: (opts: { variant: "warning"; title: string; message: string; duration: number }) => void },
): void {
  const code = parseCypherErrorCode(error)
  if (!code) return
  toast.show({
    variant: "warning",
    title: cypherErrorTitle(code),
    message: cypherErrorDescription(code),
    duration: 5000,
  })
}

/**
 * Extract the specific Cypher error code from an APIError's responseBody.
 * Returns the code string if found, undefined otherwise.
 *
 * Note: We check error.name === "APIError" directly instead of using
 * MessageV2.APIError.isInstance() to avoid a circular dependency
 * (message-v2.ts re-exports from this file).
 */
export function parseCypherErrorCode(error: ReturnType<NamedError["toObject"]>): CypherErrorCode | undefined {
  if (error.name !== "APIError") return undefined
  const responseBody = isRecord(error.data) ? error.data.responseBody : undefined
  if (typeof responseBody !== "string") return undefined
  try {
    const body = JSON.parse(responseBody)
    // Backend sends: { error: { code: "PAID_MODEL_AUTH_REQUIRED" } }
    // or: { code: "PROMOTION_MODEL_LIMIT_REACHED" }
    const code = body?.error?.code ?? body?.code
    if (typeof code === "string" && CYPHER_ERROR_CODE_VALUES.includes(code)) {
      return code as CypherErrorCode
    }
  } catch {}
  return undefined
}
