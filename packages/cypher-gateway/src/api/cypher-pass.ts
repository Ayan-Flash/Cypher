import { buildCypherHeaders } from "../headers.js"
import type { CypherPassState } from "../types.js"
import { CYPHER_API_BASE } from "./constants.js"

function record(value: unknown) {
  return value !== null && typeof value === "object" ? (value as Record<string, unknown>) : undefined
}

function num(value: unknown) {
  return typeof value === "number" && Number.isFinite(value) ? value : 0
}

export function parseCypherPassState(value: unknown): CypherPassState | null {
  const item = Array.isArray(value) ? value[0] : value
  const data = record(record(record(item)?.result)?.data)
  const root = record(data?.json) ?? data ?? record(value)
  const sub = record(root?.subscription)
  if (!sub || (sub.currentPeriodBaseCreditsUsd == null && sub.currentPeriodUsageUsd == null)) return null

  const next = sub.nextBillingAt ?? sub.nextRenewalAt
  return {
    currentPeriodBaseCreditsUsd: num(sub.currentPeriodBaseCreditsUsd),
    currentPeriodUsageUsd: num(sub.currentPeriodUsageUsd),
    currentPeriodBonusCreditsUsd: num(sub.currentPeriodBonusCreditsUsd),
    nextBillingAt: typeof next === "string" ? next : null,
  }
}

export async function fetchCypherPassState(token: string): Promise<CypherPassState | null> {
  try {
    const params = new URLSearchParams({ batch: "1", input: JSON.stringify({ "0": null }) })
    const response = await fetch(`${CYPHER_API_BASE}/api/trpc/cypherPass.getState?${params}`, {
      headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json", ...buildCypherHeaders() },
    })
    if (!response.ok) {
      console.warn(`Failed to fetch Cypher Pass: ${response.status}`)
      return null
    }
    return parseCypherPassState(await response.json())
  } catch (err) {
    console.warn("Error fetching Cypher Pass:", err)
    return null
  }
}
