import type { Model, Provider } from "@cypher/sdk/v2/client"

export function hasGateway(providers: Pick<Provider, "id">[]) {
  return providers.some((provider) => provider.id === "cypher")
}

export function visible(
  provider: Pick<Provider, "id">,
  model: Pick<Model, "mayTrainOnYourPrompts">,
  privacy: boolean,
) {
  return !privacy || provider.id !== "cypher" || model.mayTrainOnYourPrompts !== true
}
