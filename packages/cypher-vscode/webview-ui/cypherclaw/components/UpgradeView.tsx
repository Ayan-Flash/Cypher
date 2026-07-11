// CypherClaw upgrade view — shown when instance needs upgrade for chat

import { Button } from "@cypher/cypher-ui/button"
import { Card, CardTitle, CardDescription, CardActions } from "@cypher/cypher-ui/card"
import { useClaw } from "../context/claw"
import { useCypherClawLanguage } from "../context/language"

export function UpgradeView() {
  const claw = useClaw()
  const { t } = useCypherClawLanguage()

  return (
    <div class="cypherclaw-center">
      <Card class="cypherclaw-card">
        <CardTitle icon={false}>{t("cypherClaw.upgrade.title")}</CardTitle>
        <CardDescription>
          <p class="cypherclaw-card-text">{t("cypherClaw.upgrade.description1")}</p>
          <p class="cypherclaw-card-text">
            {t("cypherClaw.upgrade.description2.before")}
            <strong>{t("cypherClaw.upgrade.description2.bold")}</strong>
            {t("cypherClaw.upgrade.description2.after")}
          </p>
        </CardDescription>
        <CardActions>
          <div />
          <Button variant="primary" onClick={() => claw.openExternal("https://app.cypher.ai/claw")}>
            {t("cypherClaw.upgrade.openDashboard")}
          </Button>
        </CardActions>
      </Card>
    </div>
  )
}
