import { MessageV2 } from "@/session/message-v2"

export namespace CypherPartLifecycle {
  export const key = "cypher.lifecycle"

  export function transient(part: MessageV2.Part) {
    return part.type === "text" && part.metadata?.[key] === "transient"
  }
}
