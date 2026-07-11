---
title: "Setting Up Other Services"
description: "Configure your CypherClaw agent to use third-party tools and services that aren't pre-installed"
---

# Setting Up Other Services

While CypherClaw comes with a set of [pre-configured tool integrations](/docs/cypherclaw/tools), your agent isn't limited to just those. CypherClaw can be configured to use virtually any third-party integration as a tool — as long as it has a CLI or an API, you can teach your agent to work with it.

We have seen this pattern work well with outside services like ZenDesk, Todoist, GitLab, and more.

## If There Is a CLI

When the tool you want to integrate provides a command-line interface, follow these steps:

1. Tell CypherClaw to install the CLI.

2. Add a key, PAT, or token to the CypherClaw's [1Password](/docs/cypherclaw/tools/1password).

3. Navigate to the CypherClaw Dashboard (`app.cypher.ai/claw/settings`) > *Danger Zone* > *Edit Files* > `workspace` folder > `TOOLS.md`, and add the following to the bottom of the file:

>   TOOL is 1 SENTENCE DESCRIPTION. You have access to it via the CLI NAME CLI. The username and password are in the 1Password vault under TOOL.

4. Ask the agent to perform a task using the tool.

## If There Is No CLI, but There Is an API

When the tool only provides an API (no CLI), follow these steps:

1. Add a key, PAT, or token to the CypherClaw's [1Password](/docs/cypherclaw/tools/1password).

2. Navigate to the CypherClaw Dashboard (`app.cypher.ai/claw/settings`) > *Danger Zone* > *Edit Files* > `workspace` folder > `TOOLS.md`, and add the following to the bottom of the file:

>   TOOL is 1 SENTENCE DESCRIPTION. You have access to it via the API. API documentation is at URL OF API DOCUMENTATION. Credentials are in 1Password under TOOL NAME.

1. Ask the agent to use the API.

{% callout type="note" %}
If you have not configured your CypherClaw with the 1Password CLI, you can add the username in `TOOLS.md` and the key as an *Additional Secret* in the [CypherClaw Dashboard](https://app.cypher.ai/claw/settings) with the config path `skills.entries.<TOOL_NAME>.apiKey` and environment variable name `<TOOL_NAME>_API_KEY`.
{% /callout %}

## Improving performance

The instructions above will get your CypherClaw started with using the tool, but it will have to read the documentation every time and may fumble to use the CLI or API in question. 

As you use the CLI or API, instruct CypherClaw to do the following to make usage more reliable and less token-intensive:

* Save usage patterns to `TOOLS.md`
* Extract usage patterns into a skill
* Write a python or javascript wrapper for the CLI or API to encompass the ways you tend to use it
  
