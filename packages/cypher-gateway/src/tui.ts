/**
 * Cypher Gateway TUI Integration
 *
 * This module provides TUI-specific functionality for cypher-gateway.
 * It requires OpenCode TUI dependencies to be injected at runtime.
 *
 * Import from "@cypher/cypher-gateway/tui" for TUI features.
 */

// ============================================================================
// TUI Dependency Injection
// ============================================================================
export { initializeTUIDependencies, getTUIDependencies, areTUIDependenciesInitialized } from "./tui/context.js"
export type { TUIDependencies } from "./tui/types.js"

// ============================================================================
// TUI Helpers
// ============================================================================
export { formatProfileInfo, getOrganizationOptions, getDefaultOrganizationSelection } from "./tui/helpers.js"

// ============================================================================
// NOTE: TUI Components Moved to OpenCode
// ============================================================================
// All TUI components with JSX have been moved to packages/opencode/src/cypher/
// to ensure correct JSX transpilation with @opentui/solid.
//
// Components moved:
// - registerCypherCommands -> @/cypher/cypher-commands
// - DialogCypherTeamSelect -> @/cypher/components/dialog-cypher-team-select
// - DialogCypherOrganization -> @/cypher/components/dialog-cypher-organization
// - DialogCypherProfile -> @/cypher/components/dialog-cypher-profile
// - CypherAutoMethod -> @/cypher/components/dialog-cypher-auto-method
// - CypherNews -> @/cypher/components/cypher-news
// - NotificationBanner -> @/cypher/components/notification-banner
// - DialogCypherNotifications -> @/cypher/components/dialog-cypher-notifications
