import { z } from "zod"
import { CYPHER_API_BASE } from "./constants.js"
import { getDefaultHeaders, buildCypherHeaders } from "../headers.js"

/**
 * Cypher notification schema
 */
export const CyphercodeNotificationSchema = z.object({
  id: z.string(),
  title: z.string(),
  message: z.string(),
  action: z
    .object({
      actionText: z.string(),
      actionURL: z.string(),
    })
    .optional(),
  showIn: z.array(z.string()).optional(),
  suggestModelId: z.string().optional(),
})

export type CyphercodeNotification = z.infer<typeof CyphercodeNotificationSchema>

const NotificationsResponseSchema = z.object({
  notifications: z.array(CyphercodeNotificationSchema),
})

const NOTIFICATIONS_TIMEOUT_MS = 5000

/**
 * Fetch notifications from Cypher API
 *
 * @param options - Configuration with token and optional organization ID
 * @returns Array of notifications from the Cypher API (clients filter by showIn)
 */
export async function fetchCyphercodeNotifications(options: {
  cypherToken?: string
  cypherOrganizationId?: string
}): Promise<CyphercodeNotification[]> {
  const token = options.cypherToken
  if (!token) return []

  const url = `${CYPHER_API_BASE}/api/users/notifications`

  try {
    const response = await fetch(url, {
      headers: {
        ...getDefaultHeaders(),
        ...buildCypherHeaders(undefined, { cypherOrganizationId: options.cypherOrganizationId }),
        Authorization: `Bearer ${token}`,
      },
      signal: AbortSignal.timeout(NOTIFICATIONS_TIMEOUT_MS),
    })

    if (!response.ok) return []

    const json = await response.json()
    const result = NotificationsResponseSchema.safeParse(json)

    if (!result.success) return []

    return result.data.notifications
  } catch {
    return []
  }
}
