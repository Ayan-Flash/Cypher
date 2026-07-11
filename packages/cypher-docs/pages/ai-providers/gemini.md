---
title: "Using Google Gemini with Cypher CLI"
description: "Connect Google Gemini models to Cypher CLI. Guide to getting an API key from Google AI Studio and configuring Gemini in VS Code and the CLI."
sidebar_label: Google Gemini
---

# Using Google Gemini With Cypher CLI

Cypher CLI supports Google's Gemini family of models through the Google AI Gemini API.

**Website:** [https://ai.google.dev/](https://ai.google.dev/)

## Getting an API Key

1.  **Go to Google AI Studio:** Navigate to [https://ai.google.dev/](https://ai.google.dev/).
2.  **Sign In:** Sign in with your Google account.
3.  **Create API Key:** Click on "Create API key" in the left-hand menu.
4.  **Copy API Key:** Copy the generated API key.

## Configuration in Cypher CLI

{% tabs %}
{% tab label="VSCode" %}

Open **Settings** (gear icon) and go to the **Providers** tab to add Google Gemini and enter your API key.

The extension stores this in your `cypher.json` config file. You can also edit the config file directly — see the **CLI** tab for the file format.

{% /tab %}
{% tab label="CLI" %}

Set the API key as an environment variable or configure it in your `cypher.json` config file:

**Environment variable:**

```bash
export GOOGLE_GENERATIVE_AI_API_KEY="your-api-key"
```

**Config file** (`~/.config/cypher/cypher.json` or `./cypher.json`):

```jsonc
{
  "provider": {
    "google": {
      "env": ["GOOGLE_GENERATIVE_AI_API_KEY"],
    },
  },
}
```

Then set your default model:

```jsonc
{
  "model": "google/gemini-2.5-pro",
}
```

{% /tab %}
{% /tabs %}

## Tips and Notes

- **Pricing:** Gemini API usage is priced based on input and output tokens. Refer to the [Gemini pricing page](https://ai.google.dev/pricing) for detailed information.
- **Codebase Indexing:** The `gemini-embedding-001` model is specifically supported for [codebase indexing](/docs/customize/context/codebase-indexing), providing high-quality embeddings for semantic code search.
