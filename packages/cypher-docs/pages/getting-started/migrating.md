---
title: "Migrating from Cursor/Windsurf"
description: "Guide for migrating to Cypher CLI from other AI coding tools"
---

# Migrating from Cursor or Windsurf

Quickly migrate your custom rules from Cursor or Windsurf to Cypher CLI. The process typically takes just a few minutes per project.

{% callout type="info" title="Two Workflow Approaches"%}

Cypher CLI supports **two complementary workflows**—choose the one that fits your style, or use both:

1. **Autocomplete (Ghost)**: Tab-to-accept inline suggestions as you type, similar to Cursor and Windsurf. Enable via Settings → Ghost.
2. **Chat-driven**: Describe what you want in the chat panel and the AI generates complete implementations.

Many developers combine both approaches: autocomplete for quick completions while typing, and chat for larger refactors or multi-file changes. See [Choosing Your Workflow](#choosing-your-workflow) for details.

{% /callout %}

## Why Cypher CLI's Rules System?

Cypher CLI simplifies AI configuration while adding powerful new capabilities:

- **Simple format**: Plain Markdown files—no YAML frontmatter or GUI configuration required
- **Mode-specific rules**: Different rules for different workflows (Code, Debug, Ask, custom modes)
- **Better version control**: All configuration lives in your repository as readable Markdown
- **More control**: Custom modes let you define specialized workflows with their own rules and permissions

## Quick Migration Guide

Choose your current tool:

- [Migrating from Cursor](#migrating-from-cursor) → Skip to Cursor migration
- [Migrating from Windsurf](#migrating-from-windsurf) → Skip to Windsurf migration

## Migrating from Cursor

### What's Different in Cypher CLI

| Cursor | Cypher CLI | Key Difference |
|---|---|---|
| `.cursor/rules/*.mdc` with YAML frontmatter | `.cypher/rules/*.md` plain Markdown | No YAML metadata required |
| `alwaysApply: true/false` metadata | File location determines scope | Scope controlled by directory structure |
| `globs: ["*.ts"]` for file patterns | Mode-specific directories or custom modes | File patterns handled via custom modes |
| `description` for AI activation | Clear file names and organization | Relies on explicit file organization |
| Global rules in UI settings | `~/.cypher/rules/*.md` files | Global rules stored as files in home folder |

### Migration Steps

**1. Identify your rules:**

```bash
ls -la .cursor/rules/        # Project rules
ls -la .cursorrules          # Legacy file (if present)
```

**2. Create Cypher CLI directory:**

```bash
mkdir -p .cypher/rules
```

**3. Convert `.mdc` files to `.md`:**

For each file in `.cursor/rules/`, remove the YAML frontmatter and keep just the Markdown content.

**Cursor format:**

```mdc
---
description: TypeScript coding standards
globs: ["*.ts", "*.tsx"]
alwaysApply: false
---

# TypeScript Standards
- Always use TypeScript for new files
- Prefer functional components in React
```

**Cypher CLI format:**

```markdown
# TypeScript Standards

- Always use TypeScript for new files
- Prefer functional components in React
```

**4. Migrate in one command:**

```bash
# Copy all files
for file in .cursor/rules/*.mdc; do
  basename="${file##*/}"
  cp "$file" ".cypher/rules/${basename%.mdc}.md"
done

# Then manually edit each file to remove YAML frontmatter (the --- section at the top)
```

**5. Migrate global rules:**

- Open `Cursor Settings → General → Rules for AI`
- Copy the text content
- Save to `~/.cypher/rules/cursor-global.md`

**6. Handle legacy `.cursorrules`:**

```bash
cp .cursorrules .cypher/rules/legacy-rules.md
```

### Converting Cursor's `globs` Patterns

Cursor's `globs` field specifies which files a rule applies to. Cypher CLI handles this through **mode-specific directories** instead.

**Cursor approach:**

```mdc
---
globs: ["*.ts", "*.tsx"]
---
Rules for TypeScript files...
```

**Cypher CLI approach (Option 1 - Mode-specific directory):**

```bash
mkdir -p .cypher/rules-code
# Save TypeScript-specific rules here
```

**Cypher CLI approach (Option 2 - Custom mode):**

```yaml
# .cyphermodes (at project root)
- slug: typescript
  name: TypeScript
  roleDefinition: You work on TypeScript files
  groups:
    - read
    - [edit, { fileRegex: '\\.tsx?$' }]
    - ask
```

Then place rules in `.cypher/rules-typescript/`

### Flattening Nested Cursor Rules

Cursor supports nested `.cursor/rules/` directories. Cypher CLI uses flat structure with descriptive names:

```bash
# Cursor: .cursor/rules/backend/server/api-rules.mdc
# Cypher CLI: .cypher/rules/backend-server-api-rules.md
```

## Migrating from Windsurf

### What's Different in Cypher CLI

| Windsurf | Cypher CLI | Key Difference |
|---|---|---|
| `.windsurf/rules/*.md` | `.cypher/rules/*.md` | Same Markdown format |
| GUI configuration for activation modes | File location determines scope | Scope controlled by directory structure |
| "Always On" mode (GUI) | Place in `.cypher/rules/` | Rules stored as files, not GUI settings |
| "Glob" mode (GUI) | Mode-specific directories | File patterns handled via mode directories |
| 12,000 character limit per rule | No hard limit | No character limit on rule files |
| Global rules in `~/.codeium/windsurf/memories/global_rules.md` | `~/.cypher/rules/*.md` | Global rules in home folder, multiple files |

### Migration Steps

**1. Identify your rules:**

```bash
ls -la .windsurf/rules/      # Project rules
ls -la .windsurfrules        # Legacy file (if present)
```

**2. Create Cypher CLI directory:**

```bash
mkdir -p .cypher/rules
```

**3. Copy files directly** (already Markdown):

```bash
cp .windsurf/rules/*.md .cypher/rules/
```

**4. Migrate global rules:**

```bash
cp ~/.codeium/windsurf/memories/global_rules.md ~/.cypher/rules/global-rules.md
```

**5. Handle legacy `.windsurfrules`:**

```bash
cp .windsurfrules .cypher/rules/legacy-rules.md
```

**6. Split large rules if needed:**

If you had rules approaching the 12,000 character limit, split them:

```bash
# Instead of one large file:
# .windsurf/rules/all-conventions.md (11,500 chars)

# Split into focused files:
# .cypher/rules/api-conventions.md
# .cypher/rules/testing-standards.md
# .cypher/rules/code-style.md
```

### Converting Windsurf's Activation Modes

Windsurf configures activation through the GUI. In Cypher CLI, file organization replaces GUI configuration:

| Windsurf GUI Mode | Cypher CLI Equivalent |
|---|---|
| **Always On** | Place in `.cypher/rules/` (default) |
| **Glob** (file patterns) | Mode-specific directory or custom mode |
| **Model Decision** | Clear file names by concern (e.g., `testing-guidelines.md`) |
| **Manual** | Organize with descriptive names |

**Example - Converting a Glob rule:**

If you had a rule in Windsurf with Glob mode set to `*.test.ts`, create a custom test mode:

```yaml
# .cyphermodes (at project root)
- slug: test
  name: Testing
  roleDefinition: You write and maintain tests
  groups:
    - read
    - [edit, { fileRegex: '\\.(test|spec)\\.(ts|js)$' }]
    - ask
```

Then place the rule in `.cypher/rules-test/`

## AGENTS.md Support

All three tools support the `AGENTS.md` standard. If you have one, it works in Cypher CLI automatically:

```bash
# Verify it exists
ls -la AGENTS.md

# That's it - Cypher CLI loads it automatically (enabled by default)
```

**Important:** Use uppercase `AGENTS.md` (not `agents.md`). Cypher CLI also accepts `AGENT.md` (singular) as a fallback.

**Note:** Both `AGENTS.md` and `AGENT.md` are write-protected files in Cypher CLI and require user approval to modify.

## Understanding Mode-Specific Rules

This is Cypher CLI's unique feature that replaces both Cursor's `globs` and Windsurf's activation modes.

### Directory Structure

```bash
.cypher/rules/              # Apply to ALL modes
.cypher/rules-code/         # Only in Code mode
.cypher/rules-debug/        # Only in Debug mode
.cypher/rules-ask/          # Only in Ask mode
.cypher/rules-{custom}/     # Only in your custom mode
```

### Real-World Example

**From Cursor:**

```mdc
---
description: Testing best practices
globs: ["**/*.test.ts", "**/*.spec.ts"]
---
# Testing Rules
- Write tests for all features
- Maintain >80% coverage
```

**To Cypher CLI:**

```bash
# 1. Create test mode directory
mkdir -p .cypher/rules-test

# 2. Save rule as plain Markdown
cat > .cypher/rules-test/testing-standards.md << 'EOF'
# Testing Rules
- Write tests for all features
- Maintain >80% coverage
EOF

# 3. Define the mode (optional - creates a custom mode)
# Add to .cypher/config.yaml:
# modes:
#   - slug: test
#     name: Test Mode
#     groups: [read, edit, ask]
```

## Post-Migration Checklist

After migration:

- [ ] **Verify rules loaded:** Click law icon (⚖️) in Cypher CLI panel
- [ ] **Test rule application:** Ask Cypher CLI to perform tasks following your rules
- [ ] **Organize rules:** Split large files, use clear names
- [ ] **Set up mode-specific rules:** Create directories for specialized workflows
- [ ] **Update team docs:** Document new `.cypher/rules/` location
- [ ] **Commit to version control:** `git add .cypher/`
- [ ] **Remove old directories:** Delete `.cursor/` or `.windsurf/` folders once verified
- [ ] **Set up autocomplete:** If you used Cursor/Windsurf autocomplete, enable Ghost (Settings → Ghost) for the same Tab-to-accept experience

## Troubleshooting

### Rules Not Appearing

**Check file location:**

```bash
ls -la .cypher/rules/      # Project rules
ls -la ~/.cypher/rules/    # Global rules
```

**Verify file format:**

- Can be any text file extension (`.md`, `.txt`, etc.) - binary files are automatically filtered out
- Remove all YAML frontmatter from Cursor files
- Ensure files are not cache/temp files (`.cache`, `.tmp`, `.log`, `.bak`, etc.)

**Reload VS Code:**

- `Cmd+R` (Mac) or `Ctrl+R` (Windows/Linux)
- Or: Command Palette → "Developer: Reload Window"

### Cursor Metadata Lost

Cursor's `globs`, `alwaysApply`, and `description` don't transfer automatically. Solutions:

- **For file patterns:** Use mode-specific directories or custom modes
- **For always-on rules:** Place in `.cypher/rules/`
- **For context-specific rules:** Use clear file names and organization

### Windsurf Activation Modes Lost

Windsurf's GUI activation modes (Always On/Glob/Model Decision/Manual) aren't stored in files. Solutions:

- **Before migrating:** Document each rule's activation mode
- **After migrating:** Organize files accordingly in Cypher CLI

### Nested Rules Flattened

Cursor's nested directories don't map to Cypher CLI. Flatten with descriptive names:

```bash
# Bad: .cursor/rules/backend/api/rules.mdc
# Good: .cypher/rules/backend-api-rules.md
```

### AGENTS.md Not Loading

- **Verify filename:** Must be `AGENTS.md` or `AGENT.md` (uppercase)
- **Check location:** Must be at project root
- **Check setting:** Verify "Use Agent Rules" is enabled in Cypher CLI settings (enabled by default)
- **Reload:** Restart VS Code if needed

### Choosing Your Workflow

Cypher CLI supports **both autocomplete and chat-driven workflows**. Choose the approach that fits your coding style, or combine them:

**Autocomplete (Ghost) — Tab-to-accept inline suggestions:**

1. Open Settings → Ghost
2. Enable Ghost autocomplete
3. Configure your preferred model for completions
4. Start typing and press Tab to accept suggestions

This works the same way as Cursor and Windsurf's autocomplete. Ghost provides context-aware suggestions as you type.

**Chat-driven — describe what you want:**

- Open the chat panel and describe your intent: "Add error handling to this function" or "Create a React component for user profiles"
- The AI generates complete implementations, refactors, or fixes
- Review and approve changes before they're applied

**Combining both workflows:**

Many developers use both approaches together:

- **Autocomplete** for quick completions while writing new code
- **Chat** for larger refactors, bug fixes, or multi-file changes

There's no "right" workflow—use whatever helps you code faster

## Advanced: Creating Custom Modes

For complex workflows, define custom modes with their own rules and permissions:

```yaml
# .cyphermodes (at project root)
- slug: review
  name: Code Review
  roleDefinition: You review code and suggest improvements
  groups:
    - read
    - ask
    # Note: No edit permission - review mode is read-only

- slug: docs
  name: Documentation
  roleDefinition: You write and maintain documentation
  groups:
    - read
    - [edit, { fileRegex: '\\.md$', description: "Markdown files only" }]
    - ask
```

Then create corresponding rule directories:

```bash
mkdir -p .cypher/rules-review
mkdir -p .cypher/rules-docs
```

**Note:** `.cyphermodes` can be in YAML (preferred) or JSON format. For global modes, edit the `custom_modes.yaml` file via Settings > Edit Global Modes.

## Next Steps

- [Learn about Custom Rules](/docs/customize/custom-rules)
- [Explore Custom Modes](/docs/customize/custom-modes)
- [Set up Custom Instructions](/docs/customize/custom-instructions)
- [Join our Discord](https://cypher.ai/discord) for migration support

## Additional Resources

### Community Examples

**Cursor users:**

- [awesome-cursorrules](https://github.com/PatrickJS/awesome-cursorrules) - 700+ examples you can adapt

**Windsurf users:**

- [Official Rules Directory](https://windsurf.com/editor/directory)
- [windsurfrules](https://github.com/kinopeee/windsurfrules)

**Cross-tool:**

- [AGENTS.md Specification](https://agents.md)
- [dotagent](https://github.com/johnlindquist/dotagent) - Universal converter tool
