import { CyphercodeMarkdown } from "../config/markdown"

export namespace CyphercodeInstruction {
  export function content(text: string, item: string) {
    return CyphercodeMarkdown.substitute(text, item)
  }
}
