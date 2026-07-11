// Cypher notification types (mirrored from cypher-gateway)
export interface CyphercodeNotificationAction {
  actionText: string
  actionURL: string
}

export interface CyphercodeNotification {
  id: string
  title: string
  message: string
  action?: CyphercodeNotificationAction
  showIn?: string[]
  suggestModelId?: string
}

// Profile types from cypher-gateway
export interface CyphercodeBalance {
  balance: number
}

export interface CypherPassState {
  currentPeriodBaseCreditsUsd: number
  currentPeriodUsageUsd: number
  currentPeriodBonusCreditsUsd: number
  nextBillingAt?: string | null
}

export interface ProfileData {
  profile: {
    email: string
    name?: string
    organizations?: Array<{ id: string; name: string; role: string }>
    selectedOrganizationId?: string
    hasPersonalAccount?: boolean
  }
  balance: CyphercodeBalance | null
  cypherPass: CypherPassState | null
  currentOrgId: string | null
}
