import { createContext, createSignal, onCleanup, useContext, type Accessor, type ParentComponent } from "solid-js"
import {
  EMPTY_CYPHER_EMBEDDING_MODEL_CATALOG,
  type CypherEmbeddingModelCatalog,
} from "@cypher/cypher-indexing/embedding-models"
import { useVSCode } from "./vscode"
import type { ExtensionMessage } from "../types/messages"

type CypherEmbeddingModelsContextValue = {
  catalog: Accessor<CypherEmbeddingModelCatalog>
}

export const CypherEmbeddingModelsContext = createContext<CypherEmbeddingModelsContextValue>()

export const CypherEmbeddingModelsProvider: ParentComponent = (props) => {
  const vscode = useVSCode()
  const [catalog, setCatalog] = createSignal<CypherEmbeddingModelCatalog>(EMPTY_CYPHER_EMBEDDING_MODEL_CATALOG)

  const unsubscribe = vscode.onMessage((message: ExtensionMessage) => {
    if (message.type !== "cypherEmbeddingModelsLoaded") return
    setCatalog(message.catalog)
  })

  vscode.postMessage({ type: "requestCypherEmbeddingModels" })

  onCleanup(unsubscribe)

  return <CypherEmbeddingModelsContext.Provider value={{ catalog }}>{props.children}</CypherEmbeddingModelsContext.Provider>
}

export function useCypherEmbeddingModels(): CypherEmbeddingModelsContextValue {
  const context = useContext(CypherEmbeddingModelsContext)
  if (!context) {
    throw new Error("useCypherEmbeddingModels must be used within a CypherEmbeddingModelsProvider")
  }
  return context
}
