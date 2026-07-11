import { NavSection } from "../types"

export const CypherClawNav: NavSection[] = [
  {
    title: "CypherClaw",
    links: [
      { href: "/cypherclaw/overview", children: "Overview" },
      { href: "/cypherclaw/dashboard", children: "Dashboard" },
      { href: "/cypherclaw/pre-installed-software", children: "Pre-installed Software" },
      { href: "/cypherclaw/end-to-end", children: "End to End Config" },
      {
        href: "/cypherclaw/control-ui/overview",
        children: "Control UI",
        subLinks: [
          { href: "/cypherclaw/control-ui/changing-models", children: "Changing Models" },
          { href: "/cypherclaw/control-ui/exec-approvals", children: "Exec Approvals" },
          { href: "/cypherclaw/control-ui/version-pinning", children: "Version Pinning" },
        ],
      },
      {
        href: "/cypherclaw/chat-platforms",
        children: "Chat Platforms",
        subLinks: [
          { href: "/cypherclaw/chat-platforms/telegram", children: "Telegram" },
          { href: "/cypherclaw/chat-platforms/discord", children: "Discord" },
          { href: "/cypherclaw/chat-platforms/slack", children: "Slack" },
        ],
      },
      {
        href: "/cypherclaw/development-tools",
        children: "Integrations",
        subLinks: [
          { href: "/cypherclaw/development-tools/github", children: "GitHub" },
          { href: "/cypherclaw/development-tools/google", children: "Google Workspace" },
          { href: "/cypherclaw/development-tools/linear", children: "Linear" },
          { href: "/cypherclaw/development-tools/composio", children: "Composio" },
          { href: "/cypherclaw/tools/1password", children: "1Password" },
          { href: "/cypherclaw/tools/brave-search", children: "Brave Search" },
          { href: "/cypherclaw/tools/agentcard", children: "AgentCard" },
          { href: "/cypherclaw/tools/other-tools", children: "Other Tools" },
        ],
      },
      {
        href: "/cypherclaw/triggers",
        children: "Triggers",
        subLinks: [
          { href: "/cypherclaw/triggers/webhooks", children: "Webhooks" },
          { href: "/cypherclaw/triggers/scheduled", children: "Scheduled" },
        ],
      },
      {
        href: "/cypherclaw/troubleshooting/common-questions",
        children: "Troubleshooting",
        subLinks: [
          { href: "/cypherclaw/troubleshooting/common-questions", children: "Common Questions" },
          { href: "/cypherclaw/troubleshooting/gateway-process", children: "Gateway Process States" },
          { href: "/cypherclaw/troubleshooting/architecture", children: "Architecture Notes" },
        ],
      },
      {
        href: "/cypherclaw/faq/general",
        children: "FAQ",
        subLinks: [
          { href: "/cypherclaw/faq/general", children: "General" },
          { href: "/cypherclaw/faq/pricing", children: "Pricing" },
        ],
      },
    ],
  },
]
