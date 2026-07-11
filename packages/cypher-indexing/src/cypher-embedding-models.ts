export type CypherEmbeddingModel = {
  id: string
  name: string
  dimension: number
  scoreThreshold: number
  note?: string
}

export type CypherEmbeddingModelCatalog = {
  defaultModel: string
  models: CypherEmbeddingModel[]
  aliases: Record<string, string>
}

export const EMPTY_CYPHER_EMBEDDING_MODEL_CATALOG: CypherEmbeddingModelCatalog = {
  defaultModel: "",
  models: [],
  aliases: {},
}

export function normalizeCypherEmbeddingModelId(model: string | undefined, catalog = EMPTY_CYPHER_EMBEDDING_MODEL_CATALOG) {
  if (!model) return undefined
  return catalog.aliases[model] ?? model
}

export function getCypherEmbeddingModel(model: string | undefined, catalog = EMPTY_CYPHER_EMBEDDING_MODEL_CATALOG) {
  const id = normalizeCypherEmbeddingModelId(model, catalog)
  return catalog.models.find((item) => item.id === id)
}

export function formatCypherEmbeddingModelLabel(model: CypherEmbeddingModel): string {
  const note = model.note ? `${model.note}, ` : ""
  return `${model.name} (${note}${model.dimension}d)`
}
