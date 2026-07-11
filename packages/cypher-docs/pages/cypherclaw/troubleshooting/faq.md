---
title: "FAQ"
description: "Frequently asked questions about CypherClaw"
---

# FAQ

## How can I change my model?

You can change the model in two ways:

- **From chat** — Type `/model` in the Chat window within the OpenClaw Control UI to switch models directly.
- **From the dashboard** — Go to [https://app.cypher.ai/claw](https://app.cypher.ai/claw), select the model you want, and click **Save**. No redeploy is needed.

## Can I access the filesystem?

You can access instance files in `/root/.openclaw/` directly from the [CypherClaw Dashboard](https://app.cypher.ai/claw). This is useful for examining or restoring config files. You can also interact with files through your OpenClaw agent using its built-in file tools.

## Can I access my CypherClaw via SSH?

For security reasons, SSH access is currently disabled for all CypherClaw instances. Our primary goal is to provide a secure environment for all users, and restricting direct SSH access is one of the many measures we take to ensure the platform remains safe and protected for everyone.

## How can I update my OpenClaw?

Do **not** click **Update Now** inside the OpenClaw Control UI — this is not supported for CypherClaw instances and may break your setup.

Updates are managed by the CypherClaw platform team to ensure stability. When a new version is available, it will be announced in the **Changelog** on your dashboard. To apply the update, click **Upgrade & Redeploy** from the [CypherClaw Dashboard](/docs/cypherclaw/dashboard#redeploy).
