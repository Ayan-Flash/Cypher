/**
 * CypherClaw VS Code extension message types.
 *
 * Defines the postMessage protocol between the extension host (Node.js)
 * and the CypherClaw webview (SolidJS). The extension host owns all network
 * connections (Cypher Chat HTTP + event-service WebSocket) and relays data
 * to the webview.
 *
 * SYNC: Shared types are mirrored in webview-ui/cypherclaw/lib/types.ts —
 * keep both in sync.
 */

// ── Instance status (CypherClaw worker) ───────────────────────────────

export type ClawStatus = {
  // `recovering` and `restoring` are transitional states the worker reports
  // while bringing an instance back from an unexpected stop or a snapshot
  // restore (cloud: `services/cypherclaw/src/index.ts`).
  status:
    | "provisioned"
    | "starting"
    | "restarting"
    | "recovering"
    | "running"
    | "stopped"
    | "destroying"
    | "restoring"
    | null
  sandboxId?: string
  flyRegion?: string
  machineSize?: { cpus: number; memory_mb: number }
  openclawVersion?: string | null
  lastStartedAt?: string | null
  lastStoppedAt?: string | null
  channelCount?: number
  secretCount?: number
  userId?: string
  botName?: string | null
}

// ── Cypher Chat token envelope (gateway response) ─────────────────────

export type ChatToken = {
  token: string
  expiresAt: string // ISO timestamp
  cypherChatUrl: string
  eventServiceUrl: string
}

// ── Cypher Chat content blocks ────────────────────────────────────────
// Mirrors `@cypher/cypher-chat` schemas. See cloud/packages/cypher-chat/src/schemas.ts.

export type ExecApprovalDecision = "allow-once" | "allow-always" | "deny"

export type TextBlock = { type: "text"; text: string }

export type ActionItem = {
  label: string
  style: "primary" | "danger" | "secondary"
  value: ExecApprovalDecision
}

export type ActionsBlock = {
  type: "actions"
  groupId: string
  actions: ActionItem[]
  resolved?: {
    value: ExecApprovalDecision
    resolvedBy: string
    resolvedAt: number
  }
}

export type ContentBlock = TextBlock | ActionsBlock

// ── Cypher Chat reactions ─────────────────────────────────────────────

export type ReactionSummary = {
  emoji: string
  count: number
  memberIds: string[]
}

// ── Cypher Chat message ───────────────────────────────────────────────

export type Message = {
  id: string
  senderId: string
  content: ContentBlock[]
  inReplyToMessageId: string | null
  updatedAt: number | null
  clientUpdatedAt: number | null
  deleted: boolean
  deliveryFailed: boolean
  reactions: ReactionSummary[]
}

// ── Conversations ───────────────────────────────────────────────────

export type ConversationListItem = {
  conversationId: string
  title: string | null
  lastActivityAt: number | null
  lastReadAt: number | null
  joinedAt: number
}

export type ConversationMember = { id: string; kind: "user" | "bot" }

export type ConversationDetail = {
  id: string
  title: string | null
  createdBy: string
  createdAt: number
  members: ConversationMember[]
}

// ── Bot / conversation status (telemetry) ───────────────────────────

export type BotStatusRecord = {
  online: boolean
  at: number
  updatedAt: number
}

export type ConversationStatusRecord = {
  conversationId: string
  contextTokens: number
  contextWindow: number
  model: string | null
  provider: string | null
  at: number
  updatedAt: number
}

// ── Typed Cypher Chat events (server → client) ───────────────────────
// Event names mirror `@cypher/cypher-chat/events`.

/**
 * Snapshot of the message that was replied to. Server includes this on
 * `message.created` so clients can render a reply preview without a follow-up
 * fetch. `deleted` mirrors the soft-deletion state at the time of replying.
 */
export type ReplyToSnapshot = {
  messageId: string
  senderId: string
  content: ContentBlock[]
  deleted?: boolean
}

export type MessageCreatedEvent = {
  messageId: string
  senderId: string
  content: ContentBlock[]
  inReplyToMessageId: string | null
  clientId?: string
  replyTo?: ReplyToSnapshot | null
}

export type MessageUpdatedEvent = {
  messageId: string
  content: ContentBlock[]
  clientUpdatedAt: number | null
}

export type MessageDeletedEvent = { messageId: string }
export type MessageDeliveryFailedEvent = { messageId: string }

export type TypingEvent = { memberId: string }
export type TypingStopEvent = { memberId: string }

export type ReactionAddedEvent = { messageId: string; memberId: string; emoji: string; operationId?: string }
export type ReactionRemovedEvent = { messageId: string; memberId: string; emoji: string; operationId?: string }

/**
 * Server fans out the full conversation snapshot on `conversation.created` so
 * clients can append to their list without a follow-up fetch. Older servers may
 * still send only the `conversationId`, so the snapshot is optional.
 */
export type ConversationCreatedEvent = {
  conversationId: string
  conversation?: ConversationListItem
}
export type ConversationRenamedEvent = { conversationId: string; title: string }
export type ConversationLeftEvent = { conversationId: string }
export type ConversationReadEvent = { conversationId: string; memberId: string; lastReadAt: number }
export type ConversationActivityEvent = { conversationId: string; lastActivityAt: number }

export type ActionExecutedEvent = {
  conversationId: string
  messageId: string
  groupId: string
  value: ExecApprovalDecision
  executedBy: string
}
export type ActionDeliveryFailedEvent = {
  conversationId: string
  messageId: string
  groupId: string
}

export type BotStatusEvent = { sandboxId: string; online: boolean; at: number }
export type ConversationStatusEvent = {
  conversationId: string
  contextTokens: number
  contextWindow: number
  model: string | null
  provider: string | null
  at: number
}

export type CypherChatEventMap = {
  "message.created": MessageCreatedEvent
  "message.updated": MessageUpdatedEvent
  "message.deleted": MessageDeletedEvent
  "message.delivery_failed": MessageDeliveryFailedEvent
  typing: TypingEvent
  "typing.stop": TypingStopEvent
  "reaction.added": ReactionAddedEvent
  "reaction.removed": ReactionRemovedEvent
  "conversation.created": ConversationCreatedEvent
  "conversation.renamed": ConversationRenamedEvent
  "conversation.left": ConversationLeftEvent
  "conversation.read": ConversationReadEvent
  "conversation.activity": ConversationActivityEvent
  "action.executed": ActionExecutedEvent
  "action.delivery_failed": ActionDeliveryFailedEvent
  "bot.status": BotStatusEvent
  "conversation.status": ConversationStatusEvent
}

export type CypherChatEventName = keyof CypherChatEventMap

// ── Webview ↔ extension state ───────────────────────────────────────

export type TypingMember = { memberId: string; at: number }

// Full state snapshot pushed to the webview
// Every phase carries `locale` so the webview can resolve translations immediately.
export type CypherClawState =
  | { phase: "loading"; locale: string }
  | { phase: "noInstance"; locale: string }
  | { phase: "needsUpgrade"; locale: string }
  | { phase: "error"; locale: string; error: string }
  | {
      phase: "ready"
      locale: string
      status: ClawStatus | null
      currentUserId: string
      sandboxId: string
      conversations: ConversationListItem[]
      hasMoreConversations: boolean
      activeConversationId: string | null
      messages: Message[]
      hasMoreMessages: boolean
      botStatus: BotStatusRecord | null
      conversationStatus: ConversationStatusRecord | null
      typingMembers: TypingMember[]
    }

// ── Messages: Webview → Extension Host ──────────────────────────────

export type CypherClawInMessage =
  | { type: "cypherclaw.ready" }
  | { type: "cypherclaw.openExternal"; url: string }
  | { type: "cypherclaw.selectConversation"; conversationId: string }
  | { type: "cypherclaw.createConversation"; title?: string }
  | { type: "cypherclaw.renameConversation"; conversationId: string; title: string }
  | { type: "cypherclaw.leaveConversation"; conversationId: string }
  | { type: "cypherclaw.loadMoreConversations" }
  | {
      type: "cypherclaw.sendMessage"
      conversationId: string
      content: ContentBlock[]
      inReplyToMessageId?: string
    }
  | { type: "cypherclaw.editMessage"; conversationId: string; messageId: string; content: ContentBlock[] }
  | { type: "cypherclaw.deleteMessage"; conversationId: string; messageId: string }
  | { type: "cypherclaw.loadMoreMessages"; conversationId: string; before: string }
  | { type: "cypherclaw.addReaction"; conversationId: string; messageId: string; emoji: string }
  | { type: "cypherclaw.removeReaction"; conversationId: string; messageId: string; emoji: string }
  | {
      type: "cypherclaw.executeAction"
      conversationId: string
      messageId: string
      groupId: string
      value: ExecApprovalDecision
    }
  | { type: "cypherclaw.sendTyping"; conversationId: string }
  | { type: "cypherclaw.sendTypingStop"; conversationId: string }
  | { type: "cypherclaw.markRead"; conversationId: string }

// ── Messages: Extension Host → Webview ──────────────────────────────

export type CypherClawOutMessage =
  | { type: "cypherclaw.state"; state: CypherClawState }
  | { type: "cypherclaw.status"; data: ClawStatus | null }
  | { type: "cypherclaw.locale"; locale: string }
  | { type: "cypherclaw.error"; error: string }
  | { type: "cypherclaw.conversations"; conversations: ConversationListItem[]; hasMore: boolean; replace: boolean }
  | { type: "cypherclaw.activeConversation"; conversationId: string | null }
  | { type: "cypherclaw.messages"; conversationId: string; messages: Message[]; hasMore: boolean; replace: boolean }
  | { type: "cypherclaw.messageOptimistic"; conversationId: string; message: Message }
  | { type: "cypherclaw.messageReplaced"; conversationId: string; pendingId: string; message: Message }
  | { type: "cypherclaw.messageRemoved"; conversationId: string; messageId: string }
  | { type: "cypherclaw.botStatus"; status: BotStatusRecord | null }
  | { type: "cypherclaw.conversationStatus"; status: ConversationStatusRecord | null }
  | { type: "cypherclaw.typing"; conversationId: string; memberId: string }
  | { type: "cypherclaw.typingStop"; conversationId: string; memberId: string }
  | { type: "fontSizeChanged"; fontSize: number }
