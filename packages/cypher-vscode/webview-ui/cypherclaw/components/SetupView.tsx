// CypherClaw setup view — shown when no instance is provisioned

import { Button } from "@cypher/cypher-ui/button"
import { Card, CardTitle, CardDescription, CardActions } from "@cypher/cypher-ui/card"
import { useClaw } from "../context/claw"
import { useCypherClawLanguage } from "../context/language"

export function SetupView() {
  const claw = useClaw()
  const { t } = useCypherClawLanguage()

  return (
    <div class="cypherclaw-center">
      <Card class="cypherclaw-card">
        <CardTitle icon={false}>{t("cypherClaw.setup.title")}</CardTitle>
        <CardDescription>
          <h3 class="cypherclaw-card-subtitle">{t("cypherClaw.setup.subtitle")}</h3>
          <p class="cypherclaw-card-text">{t("cypherClaw.setup.description1")}</p>
          <p class="cypherclaw-card-text">{t("cypherClaw.setup.description2")}</p>
        </CardDescription>
        <CardActions>
          <Button variant="ghost" onClick={() => claw.openExternal("https://cypher.ai/cypherclaw")}>
            {t("cypherClaw.setup.learnMore")}
          </Button>
          <Button variant="primary" onClick={() => claw.openExternal("https://app.cypher.ai/claw")}>
            {t("cypherClaw.setup.tryCypherClaw")}
          </Button>
        </CardActions>
      </Card>
    </div>
  )
}
