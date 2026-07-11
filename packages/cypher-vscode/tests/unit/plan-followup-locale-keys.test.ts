import { describe, expect, test } from "bun:test"
import { dict as ar } from "@cypher/cypher-i18n/ar"
import { dict as br } from "@cypher/cypher-i18n/br"
import { dict as bs } from "@cypher/cypher-i18n/bs"
import { dict as da } from "@cypher/cypher-i18n/da"
import { dict as de } from "@cypher/cypher-i18n/de"
import { dict as en } from "@cypher/cypher-i18n/en"
import { dict as es } from "@cypher/cypher-i18n/es"
import { dict as fr } from "@cypher/cypher-i18n/fr"
import { dict as it } from "@cypher/cypher-i18n/it"
import { dict as ja } from "@cypher/cypher-i18n/ja"
import { dict as ko } from "@cypher/cypher-i18n/ko"
import { dict as nl } from "@cypher/cypher-i18n/nl"
import { dict as no } from "@cypher/cypher-i18n/no"
import { dict as pl } from "@cypher/cypher-i18n/pl"
import { dict as ru } from "@cypher/cypher-i18n/ru"
import { dict as th } from "@cypher/cypher-i18n/th"
import { dict as tr } from "@cypher/cypher-i18n/tr"
import { dict as uk } from "@cypher/cypher-i18n/uk"
import { dict as zh } from "@cypher/cypher-i18n/zh"
import { dict as zht } from "@cypher/cypher-i18n/zht"

const dicts: Record<string, Record<string, string>> = {
  ar,
  br,
  bs,
  da,
  de,
  en,
  es,
  fr,
  it,
  ja,
  ko,
  nl,
  no,
  pl,
  ru,
  th,
  tr,
  uk,
  zh,
  zht,
}

const keys = [
  "plan.followup.header",
  "plan.followup.question",
  "plan.followup.answer.newSession",
  "plan.followup.answer.newSession.description",
  "plan.followup.answer.continue",
  "plan.followup.answer.continue.description",
  "plan.followup.answer.keepRefining",
  "plan.followup.answer.keepRefining.description",
]

describe("plan follow-up i18n keys", () => {
  for (const locale of Object.keys(dicts)) {
    test(`${locale} defines every plan.followup.* key`, () => {
      const d = dicts[locale]!
      for (const key of keys) {
        const value = d[key]
        expect(value, `${locale} is missing ${key}`).toBeDefined()
        expect(value, `${locale} has empty value for ${key}`).toBeTruthy()
      }
    })
  }
})
