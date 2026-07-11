// CypherClaw SolidJS webview entry point

import { render } from "solid-js/web"
import "@cypher/cypher-ui/styles"
import "./cypherclaw.css"
import { CypherClawApp } from "./CypherClawApp"

const root = document.getElementById("root")
if (root) {
  render(() => <CypherClawApp />, root)
}
