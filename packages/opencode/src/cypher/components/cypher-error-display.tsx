import { createMemo, Match, Switch, type JSX } from "solid-js"
import { SplitBorder } from "@tui/component/border"
import { useTheme } from "@tui/context/theme"
import { parseCypherErrorCode, cypherErrorTitle, cypherErrorDescription } from "@/cypher/cypher-errors"
import type { AssistantMessage } from "@cypher/sdk/v2"

interface CypherErrorBlockProps {
  error: NonNullable<AssistantMessage["error"]>
  fallback: JSX.Element
}

export function CypherErrorBlock(props: CypherErrorBlockProps) {
  const { theme } = useTheme()

  const cypherErrorCode = createMemo(() => {
    return parseCypherErrorCode(props.error)
  })

  const title = createMemo(() => {
    const code = cypherErrorCode()
    return code ? cypherErrorTitle(code) : undefined
  })

  const description = createMemo(() => {
    const code = cypherErrorCode()
    return code ? cypherErrorDescription(code) : undefined
  })

  return (
    <Switch fallback={props.fallback}>
      <Match when={cypherErrorCode()}>
        <box
          border={["left"]}
          paddingTop={1}
          paddingBottom={1}
          paddingLeft={2}
          marginTop={1}
          backgroundColor={theme.backgroundPanel}
          customBorderChars={SplitBorder.customBorderChars}
          borderColor={theme.primary}
        >
          <text fg={theme.text}>{title()}</text>
          <text fg={theme.textMuted}>{description()}</text>
          <text fg={theme.primary}>{"Run /connect or `cypher auth login` to connect to Cypher Gateway"}</text>
        </box>
      </Match>
    </Switch>
  )
}
