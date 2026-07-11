import { CYPHER_API_BASE } from "./api/constants.js"
import { getAutocompleteModel, type DirectAutocompleteProviderID } from "./autocomplete.js"

export { requestMistralFim } from "./mistral-fim-endpoint.js"

export const DIRECT_FIM_ENV: Record<DirectAutocompleteProviderID, string[]> = {
  mistral: ["MISTRAL_API_KEY"],
  inception: ["INCEPTION_API_KEY"],
}

export type FimTarget =
  | { provider: "cypher"; model: string; url: string }
  | { provider: "inception"; model: string; url: string }
  | { provider: "mistral"; model: string }

const CYPHER_FIM_URL = CYPHER_API_BASE + "/api/fim/completions"
const INCEPTION_FIM_URL = "https://api.inceptionlabs.ai/v1/fim/completions"

function cypherTarget(model?: string): FimTarget {
  return { provider: "cypher", model: model ?? "mistralai/codestral-2501", url: CYPHER_FIM_URL }
}

export function resolveFimTarget(provider?: string, model?: string): FimTarget {
  if (!provider || provider === "cypher") return cypherTarget(model)

  const info = getAutocompleteModel(provider, model)
  if (info.directProvider === "mistral") {
    return { provider: "mistral", model: info.requestModel }
  }
  if (info.directProvider === "inception") {
    return { provider: "inception", model: info.requestModel, url: INCEPTION_FIM_URL }
  }
  return cypherTarget(model)
}
