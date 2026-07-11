import { TextAttributes } from "@opentui/core"
import { useKeyboard, useTerminalDimensions } from "@opentui/solid"
import * as Clipboard from "@tui/util/clipboard"
import { createSignal } from "solid-js"
import { InstallationVersion } from "@opencode-ai/core/installation/version"
import { getScrollAcceleration } from "../util/scroll"

// cypher_change start — guard against missing renderer context in ErrorBoundary fallback
function tryUseTerminalDimensions() {
  try {
    return useTerminalDimensions()
  } catch {
    return undefined
  }
}
// cypher_change end

export function ErrorComponent(props: {
  error: Error
  reset: () => void
  exit: () => Promise<void>
  mode?: "dark" | "light"
}) {
  // cypher_change start — guard against missing renderer context in ErrorBoundary fallback
  const term = tryUseTerminalDimensions()
  const height = () => term?.().height ?? process.stdout.rows ?? 24

  try {
    useKeyboard((evt) => {
      if (evt.ctrl && evt.name === "c") {
        void props.exit()
      }
    })
  } catch {
    // Keyboard handler unavailable — renderer context may be missing.
    // Ctrl+C will still work via the default signal handler.
  }
  // cypher_change end
  const [copied, setCopied] = createSignal(false)

  const issueURL = new URL("https://github.com/Cypher-Org/cypher/issues/new?template=bug-report.yml") // cypher_change

  // Choose safe fallback colors per mode since theme context may not be available
  const isLight = props.mode === "light"
  const colors = {
    bg: isLight ? "#ffffff" : "#0a0a0a",
    text: isLight ? "#1a1a1a" : "#eeeeee",
    muted: isLight ? "#8a8a8a" : "#808080",
    primary: isLight ? "#3b7dd8" : "#fab283",
  }

  if (props.error.message) {
    issueURL.searchParams.set("title", `opentui: fatal: ${props.error.message}`)
  }

  if (props.error.stack) {
    issueURL.searchParams.set(
      "description",
      "```\n" + props.error.stack.substring(0, 6000 - issueURL.toString().length) + "...\n```",
    )
  }

  issueURL.searchParams.set("cypher-version", InstallationVersion) // cypher_change

  const copyIssueURL = () => {
    void Clipboard.copy(issueURL.toString()).then(() => {
      setCopied(true)
    })
  }

  return (
    <box flexDirection="column" gap={1} backgroundColor={colors.bg}>
      <box flexDirection="row" gap={1} alignItems="center">
        <text attributes={TextAttributes.BOLD} fg={colors.text}>
          Please report an issue.
        </text>
        <box onMouseUp={copyIssueURL} backgroundColor={colors.primary} padding={1}>
          <text attributes={TextAttributes.BOLD} fg={colors.bg}>
            Copy issue URL (exception info pre-filled)
          </text>
        </box>
        {copied() && <text fg={colors.muted}>Successfully copied</text>}
      </box>
      <box flexDirection="row" gap={2} alignItems="center">
        <text fg={colors.text}>A fatal error occurred!</text>
        <box onMouseUp={props.reset} backgroundColor={colors.primary} padding={1}>
          <text fg={colors.bg}>Reset TUI</text>
        </box>
        <box onMouseUp={() => void props.exit()} backgroundColor={colors.primary} padding={1}>
          <text fg={colors.bg}>Exit</text>
        </box>
      </box>
      <scrollbox
        height={Math.floor(height() * 0.7)} // cypher_change — use safe terminal height fallback
        scrollAcceleration={getScrollAcceleration()}
      >
        <text fg={colors.muted}>{props.error.stack}</text>
      </scrollbox>
      <text fg={colors.text}>{props.error.message}</text>
    </box>
  )
}
