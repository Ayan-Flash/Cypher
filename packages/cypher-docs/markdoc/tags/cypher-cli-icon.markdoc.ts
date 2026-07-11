import { CypherIcon } from "../../components"

export const cypherCodeIcon = {
  render: CypherIcon,
  selfClosing: true,
  attributes: {
    size: {
      type: String,
      default: "1.2em",
      description: "Size of the icon (CSS height value)",
    },
  },
}
