---
title: "Troubleshooting IDE Extensions"
description: "How to capture console logs and report issues with Cypher CLI"
---

# Capturing Console Logs

Providing console logs helps us pinpoint exactly what's going wrong with your installation, network, or MCP setup. This guide walks you through capturing those logs in your IDE.

## Opening Developer Tools

1. **Open the Command Palette**: Press `Ctrl+Shift+P` (Windows/Linux) or `Cmd+Shift+P` (Mac)
2. **Search for Developer Tools**: Type `Developer: Open Webview Developer Tools` and select it

## Capturing the Error

Once you have the Developer Tools console open:

1. **Clear previous logs**: Click the "Clear Console" button (🚫 icon at the top of the Console panel) to remove old messages
2. **Reproduce the issue**: Perform the action that was causing problems
3. **Check for errors**: Look at the Console tab for error messages (usually shown in red). If you suspect connection issues, also check the **Network** tab
4. **Copy the logs**: Right-click in the console and select "Save as..." or copy the relevant error messages

## SQLite database is malformed

If every prompt fails with `SQLiteError: database disk image is malformed`, Cypher CLI's local SQLite database may be corrupted. This database stores local Cypher state such as sessions and history.

### Find the database

The database location depends on where Cypher CLI is running:

| Environment | Database path |
|---|---|
| Windows | `%LOCALAPPDATA%\cypher\cypher.db` |
| macOS | `~/Library/Application Support/cypher/cypher.db` |
| Linux | `~/.local/share/cypher/cypher.db` |
| VS Code Remote SSH | `~/.local/share/cypher/cypher.db` on the remote machine |

{% callout type="warning" %}
When using VS Code Remote SSH, check the remote Linux machine, not your local Windows or macOS computer.
{% /callout %}

### Reset the database

Close VS Code or stop the Cypher backend first. On Linux or Remote SSH, run:

```bash
pkill -f "cypher serve"
mkdir -p ~/.local/share/cypher
mv ~/.local/share/cypher/cypher.db ~/.local/share/cypher/cypher.db.bak
mv ~/.local/share/cypher/cypher.db-wal ~/.local/share/cypher/cypher.db-wal.bak 2>/dev/null
mv ~/.local/share/cypher/cypher.db-shm ~/.local/share/cypher/cypher.db-shm.bak 2>/dev/null
```

Then reload VS Code or reconnect Remote SSH. Cypher CLI recreates the database the next time it starts.

On Windows or macOS, rename the database file and any `cypher.db-wal` or `cypher.db-shm` files in the same folder, then restart the IDE.

{% callout type="warning" %}
Renaming this database resets local Cypher CLI sessions and history for that machine. Keep the `.bak` files if you need to share them with support or attempt recovery later.
{% /callout %}

### Fully reset local Cypher data

If resetting the database does not fix the issue, you can fully reset Cypher CLI's local data. This also removes local configuration and cache files, so use it only after trying the database reset above.

On Linux or VS Code Remote SSH, run this on the machine where Cypher CLI is running:

```bash
pkill -f "cypher serve"
mv ~/.local/share/cypher ~/.local/share/cypher.bak 2>/dev/null
mv ~/.config/cypher ~/.config/cypher.bak 2>/dev/null
mv ~/.cache/cypher ~/.cache/cypher.bak 2>/dev/null
```

Then reload VS Code or reconnect Remote SSH. Cypher CLI recreates these directories the next time it starts.

{% callout type="warning" %}
This resets local sessions, history, settings, and cached data. Prefer renaming the directories instead of deleting them so you can recover files. Remove secrets such as API keys or tokens before sharing any backup with support.
{% /callout %}

## Contact Support

If you're unable to resolve the issue, please inspect the console logs, remove any secrets, and send the logs to **[hi@cypher.ai](mailto:hi@cypher.ai)** along with the following:

- The error messages from the console
- Steps to reproduce the issue
- Screenshots or screen recordings of the issue
- Your IDE and Cypher CLI version
