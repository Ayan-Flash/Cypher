// CypherClaw root component

import { Switch, Match } from "solid-js"
import { ThemeProvider } from "@cypher/cypher-ui/theme"
import { MarkedProvider } from "@cypher/cypher-ui/context/marked"
import { Button } from "@cypher/cypher-ui/button"
import { Spinner } from "@cypher/cypher-ui/spinner"
import { Toast } from "@cypher/cypher-ui/toast"
import { ClawProvider, useClaw } from "./context/claw"
import { CypherClawLanguageProvider, useCypherClawLanguage } from "./context/language"
import { ConversationList } from "./components/ConversationList"
import { MessageArea } from "./components/MessageArea"
import { StatusSidebar } from "./components/StatusSidebar"
import { SetupView } from "./components/SetupView"
import { UpgradeView } from "./components/UpgradeView"

function Content() {
  const claw = useClaw()
  const { t } = useCypherClawLanguage()

  return (
    <div class="cypherclaw-root">
      <Switch>
        <Match when={claw.phase() === "loading"}>
          <div class="cypherclaw-center">
            <div class="cypherclaw-loading">
              <Spinner />
              <span>{t("cypherClaw.loading")}</span>
            </div>
          </div>
        </Match>
        <Match when={claw.phase() === "noInstance"}>
          <SetupView />
        </Match>
        <Match when={claw.phase() === "needsUpgrade"}>
          <UpgradeView />
        </Match>
        <Match when={claw.phase() === "error"}>
          <div class="cypherclaw-center">
            <div class="cypherclaw-error-view">
              <span class="cypherclaw-error-text">{claw.error()}</span>
              <Button variant="primary" onClick={() => claw.retry()}>
                {t("cypherClaw.error.retry")}
              </Button>
            </div>
          </div>
        </Match>
        <Match when={claw.phase() === "ready"}>
          <div class="cypherclaw-layout">
            <ConversationList />
            <MessageArea />
            <StatusSidebar />
          </div>
        </Match>
      </Switch>
      <Toast.Region />
    </div>
  )
}

export function CypherClawApp() {
  return (
    <ThemeProvider defaultTheme="cypher-vscode">
      <ClawProvider>
        <LanguageBridge>
          <MarkedProvider>
            <Content />
          </MarkedProvider>
        </LanguageBridge>
      </ClawProvider>
    </ThemeProvider>
  )
}

/** Bridges the claw context locale into the language provider. Must be below ClawProvider. */
function LanguageBridge(props: { children: any }) {
  const claw = useClaw()
  return <CypherClawLanguageProvider locale={claw.locale}>{props.children}</CypherClawLanguageProvider>
}
