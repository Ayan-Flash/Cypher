import React from "react"
import { Icon } from "./Icon"

interface CypherIconProps {
  size?: string
}

export function CypherIcon({ size = "1.2em" }: CypherIconProps) {
  return <Icon src="/docs/img/cypher-v1.svg" srcDark="/docs/img/cypher-v1-white.svg" alt="Cypher CLI Icon" size={size} />
}
