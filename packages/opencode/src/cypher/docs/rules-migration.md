# Cyphercode Rules Migration

This document explains how Cyphercode rules are automatically migrated to Opencode's `instructions` config array.

## Overview

Cyphercode stores rules in various file locations. When Opencode starts, it reads these files and injects their paths into the `instructions` config array, which Opencode then loads as part of the system prompt.

## Key Guarantees

### 1. Read-Only Migration

The migration **never modifies project files**. We only:

- Read existing rule files from disk
- Inject file paths into the config's `instructions` array
- Never write to the project or modify any files

### 2. Combines with Existing Config (Never Overwrites)

If you have existing opencode config with `instructions`, the Cyphercode rules are **combined**, not replaced:

```typescript
// Example: User has opencode.json with:
{ "instructions": ["AGENTS.md", "custom-rules.md"] }

// Cyphercode rules add:
{ "instructions": [".cypherrules", ".cypher/rules/coding.md"] }

// Result (combined, deduplicated):
{ "instructions": ["AGENTS.md", "custom-rules.md", ".cypherrules", ".cypher/rules/coding.md"] }
```

### 3. Restart to Pick Up Changes

If you change your Cyphercode configuration (e.g., edit `.cypherrules`), simply restart cypher-cli to pick up the new config. No manual migration or conversion needed.

## Source Locations

The migrator reads rules from these locations:

### Project Rules

| Location | Description |
|---|---|
| `.cypherrules` | Legacy single-file rules in project root |
| `.cypher/rules/*.md` | Directory-based rules (multiple markdown files) |
| `.cypherrules-{mode}` | Mode-specific legacy rules (e.g., `.cypherrules-code`) |
| `.cypher/rules-{mode}/*.md` | Mode-specific rule directories |

### Global Rules

| Location | Description |
|---|---|
| `~/.cypher/rules/*.md` | Global rules directory |

## File Mapping

| Cyphercode Location | Opencode Equivalent |
|---|---|
| `.cypherrules` | `instructions: [".cypherrules"]` |
| `.cypherrules-{mode}` | `instructions: [".cypherrules-{mode}"]` |
| `.cypher/rules/*.md` | `instructions: [".cypher/rules/file.md", ...]` |
| `.cypher/rules-{mode}/*.md` | `instructions: [".cypher/rules-{mode}/file.md", ...]` |
| `~/.cypher/rules/*.md` | `instructions: ["~/.cypher/rules/file.md", ...]` |

## AGENTS.md Compatibility

`AGENTS.md` is loaded **natively** by Opencode - no migration needed. Opencode automatically loads:

- `AGENTS.md` in project root
- `CLAUDE.md` in project root
- `~/.config/cypher/AGENTS.md` (global)

## Not Migrated

The following are **not** migrated:

- `.roorules` - Roo-specific rules
- `.clinerules` - Cline-specific rules

Only Cyphercode-specific files (`.cypherrules`, `.cypher/rules/`) are migrated.

## Mode-Specific Rules

Mode-specific rules (e.g., `.cypherrules-code`, `.cypher/rules-architect/`) are included by default. All mode-specific rules are loaded regardless of the current mode.

## Warnings

The migrator generates warnings for:

- **Legacy files**: When `.cypherrules` is found, a warning suggests migrating to `.cypher/rules/` directory structure

## Example

### Before (Cyphercode)

```
project/
├── .cypherrules           # Legacy rules
├── .cypherrules-code      # Code-mode specific
└── .cypher/
    └── rules/
        ├── coding.md        # Coding standards
        └── testing.md       # Testing guidelines
```

### After (Opencode Config)

```json
{
  "instructions": [
    "/path/to/project/.cypher/rules/coding.md",
    "/path/to/project/.cypher/rules/testing.md",
    "/path/to/project/.cypherrules",
    "/path/to/project/.cypherrules-code"
  ]
}
```

## Troubleshooting

### Rules not appearing

1. Check the file exists at the expected location
2. Ensure markdown files have `.md` extension
3. Restart cypher-cli to pick up changes

### Duplicate rules

The `mergeConfigConcatArrays` function automatically deduplicates the `instructions` array using `Array.from(new Set([...]))`.

## Related Files

- [`rules-migrator.ts`](../rules-migrator.ts) - Core migration logic
- [`config-injector.ts`](../config-injector.ts) - Config building and injection
- [`modes-migration.md`](./modes-migration.md) - Modes migration documentation
