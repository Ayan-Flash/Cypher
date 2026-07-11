/**
 * Cypher Gateway Commands for TUI
 *
 * Provides /profile and /teams commands that are only visible when connected to Cypher Gateway.
 */

import { createMemo } from "solid-js"
import { useBindings } from "@tui/keymap"
import { useSync } from "@tui/context/sync"
import { useRoute } from "@tui/context/route"
import { useDialog } from "@tui/ui/dialog"
import { useToast } from "@tui/ui/toast"
import { DialogAlert } from "@tui/ui/dialog-alert"
import type { Organization } from "@cypher/cypher-gateway"
import type { ClawStatus } from "./claw/types.js"
import { DialogCypherTeamSelect } from "./components/dialog-cypher-team-select.js"
import { DialogCypherProfile } from "./components/dialog-cypher-profile.js"
import { DialogClawSetup } from "./components/dialog-claw-setup.js"
import { DialogClawUpgrade } from "./components/dialog-claw-upgrade.js"
import { DialogIndexing } from "./components/dialog-indexing.js"
import { indexingEnabled } from "./indexing-feature.js"
import { refreshBalance } from "./balance-refresh.js"

// These types are OpenCode-internal and imported at runtime
type UseSDK = any
type SDK = any

/**
 * Register all Cypher Gateway commands
 * Call this from a component inside the TUI app
 *
 * @param useSDK - OpenCode's useSDK hook (passed from TUI context)
 */
export function registerCypherCommands(useSDK: () => UseSDK) {
  const sync = useSync()
  const route = useRoute()
  const dialog = useDialog()
  const sdk = useSDK()
  const toast = useToast()

  // Only show Cypher commands when connected to Cypher Gateway
  const isCypherConnected = createMemo(() => {
    return sync.data.provider_next.connected.includes("cypher")
  })
  const indexing = createMemo(() => indexingEnabled(sync.data.config))

  useBindings(() => ({
    commands: [
      // /cypherclaw command
      {
        name: "cypher.claw",
        title: "CypherClaw",
        desc: "Open CypherClaw chat & dashboard",
        category: "Cypher",
        slashName: "cypherclaw",
        slashAliases: ["claw"],
        enabled: isCypherConnected(),
        hidden: !isCypherConnected(),
        run: async () => {
          // Fetch profile (for org context) and instance status in parallel
          const [profileRes, res] = await Promise.all([
            sdk.client.cypher.profile().catch(() => null),
            sdk.client.cypher.claw.status().catch(() => null),
          ])
          const orgId = profileRes?.data?.currentOrgId ?? null
          const status = res?.data as ClawStatus | undefined

          // No instance provisioned
          if (!status || !status.userId || res.error) {
            dialog.replace(() => <DialogClawSetup orgId={orgId} />)
            return
          }

          // Instance exists — check for chat credentials
          const creds = await sdk.client.cypher.claw.chatCredentials().catch(() => null)

          if (!creds?.data || creds.error) {
            // Instance exists but no chat credentials — needs upgrade
            dialog.replace(() => <DialogClawUpgrade orgId={orgId} />)
            return
          }

          // Everything ready — navigate to full-screen chat view
          route.navigate({ type: "cypherclaw" })
          dialog.clear()
        },
      },

      // /remote command
      {
        name: "remote.toggle",
        title: "Toggle remote",
        desc: "Enable or disable remote session relay",
        category: "Cypher",
        slashName: "remote",
        enabled: isCypherConnected(),
        hidden: !isCypherConnected(),
        run: async () => {
          try {
            const current = await sdk.client.remote.status()

            if (current.error || !current.data) {
              dialog.replace(() => <DialogAlert title="Error" message="Failed to fetch remote status." />)
              return
            }

            if (current.data.enabled) {
              await sdk.client.remote.disable()
              toast.show({ message: "Remote disabled", variant: "success" })
            } else {
              const result = await sdk.client.remote.enable()
              if (result.error) {
                const err = result.error as { error?: string }
                const msg = err?.error ?? "Failed to enable remote."
                dialog.replace(() => <DialogAlert title="Error" message={msg} />)
                return
              }
              toast.show({ message: "Remote enabled", variant: "success" })
            }

            dialog.clear()
          } catch (error) {
            dialog.replace(() => <DialogAlert title="Error" message={`Failed to toggle remote: ${error}`} />)
          }
        },
      },

      // /profile command
      {
        name: "cypher.profile",
        title: "Profile",
        desc: "View your Cypher Gateway profile",
        category: "Cypher",
        slashName: "profile",
        slashAliases: ["me", "whoami"],
        enabled: isCypherConnected(),
        hidden: !isCypherConnected(),
        run: async () => {
          try {
            // Fetch profile and balance using server endpoint
            const response = await sdk.client.cypher.profile()

            if (response.error || !response.data) {
              dialog.replace(() => (
                <DialogAlert
                  title="Error"
                  message="Failed to fetch profile. Please ensure you're authenticated with Cypher Gateway."
                />
              ))
              return
            }

            const { profile, balance, currentOrgId } = response.data

            // Show profile dialog with clickable usage link
            dialog.replace(() => <DialogCypherProfile profile={profile} balance={balance} currentOrgId={currentOrgId} />)
          } catch (error) {
            dialog.replace(() => <DialogAlert title="Error" message={`Failed to fetch profile: ${error}`} />)
          }
        },
      },

      ...(indexing()
        ? [
          {
            name: "cypher.indexing",
            title: "Indexing",
            desc: "Configure codebase indexing",
            category: "Cypher",
            slashName: "indexing",
            slashAliases: ["index", "embedding"],
            run: () => {
              dialog.replace(() => <DialogIndexing useSDK={useSDK} />)
            },
          },
        ]
        : []),

      // /teams command
      {
        name: "cypher.teams",
        title: "Teams",
        desc: "Switch between Cypher Gateway teams",
        category: "Cypher",
        slashName: "teams",
        slashAliases: ["team", "org", "orgs"],
        enabled: isCypherConnected(),
        hidden: !isCypherConnected(),
        run: async () => {
          try {
            // Fetch profile to get organizations
            const response = await sdk.client.cypher.profile()

            if (response.error || !response.data) {
              dialog.replace(() => (
                <DialogAlert
                  title="Error"
                  message="Failed to fetch teams. Please ensure you're authenticated with Cypher Gateway."
                />
              ))
              return
            }

            const { profile, currentOrgId } = response.data

            if (!profile.organizations || profile.organizations.length === 0) {
              dialog.replace(() => (
                <DialogAlert
                  title="No Teams Available"
                  message="You're not a member of any teams.\nVisit https://app.cypher.ai to create or join a team."
                />
              ))
              return
            }

            // Show team selection dialog
            dialog.replace(() => (
              <DialogCypherTeamSelect
                organizations={profile.organizations!}
                currentOrgId={currentOrgId}
                hasPersonalAccount={profile.hasPersonalAccount !== false}
                onSelect={async (orgId) => {
                  try {
                    // Switch to team immediately using server endpoint
                    const result = await sdk.client.cypher.organization.set({
                      organizationId: orgId,
                    })
                    if (result.error) {
                      toast.show({
                        message: "Failed to switch team",
                        variant: "error",
                      })
                      dialog.clear()
                      return
                    }

                    // Refresh provider state to reload models with new organization context
                    await sdk.client.instance.dispose()
                    await sync.bootstrap()

                    // Update the sidebar balance immediately for the newly selected account
                    refreshBalance()

                    // Show success toast
                    const teamName = orgId
                      ? profile.organizations!.find((o: Organization) => o.id === orgId)?.name
                      : "Personal"

                    toast.show({
                      message: `Switched to: ${teamName}`,
                      variant: "success",
                    })

                    // Close dialog
                    dialog.clear()
                  } catch (error) {
                    if (error instanceof DOMException && error.name === "AbortError") return
                    toast.show({
                      message: "Failed to switch team",
                      variant: "error",
                    })
                    dialog.clear()
                  }
                }}
              />
            ))
          } catch (error) {
            dialog.replace(() => <DialogAlert title="Error" message={`Failed to fetch teams: ${error}`} />)
          }
        },
      },
    ].map((command) => ({
      namespace: "palette",
      ...command,
    })),
  }))
}
