---
name: cypher-merge-minimizer
description: Use when changing shared upstream-owned files to add Cypher-specific behavior, editing `cypher_change` markers in shared code, or moving additive behavior out of shared code to reduce upstream merge conflicts. Do not use for changes confined to Cypher-owned paths such as `packages/cypher-vscode/` or `packages/cypher-ui/`.
---

# Cypher Merge Minimizer

Use this skill whenever a normal development task touches shared upstream-owned code and includes Cypher-specific behavior, especially for marker cleanup, extraction work, or `cypher_change` annotations.

Do not use this skill when all changes are confined to Cypher-owned paths, including `packages/cypher-vscode/`, `packages/cypher-ui/`, and paths with `cypher` in their name. Those files are not merged from upstream and do not need merge-minimization guidance. If a task also touches shared upstream-owned code, use this skill for the shared portion only.

Do not use this as the primary guide for upstream merge resolution. Upstream merges have their own instructions and should not duplicate that workflow here.

## Goal

Minimize Cypher's long-term diff against upstream OpenCode while preserving behavior.

Prefer this shape for Cypher-specific additions:

1. Shared upstream file contains only a minimal hook, import, call, registration, or config entry.
2. Cypher-specific behavior lives in Cypher-owned code.
3. Unavoidable shared-file changes have narrow `cypher_change` markers.
4. The annotation checker passes.

For changes to existing upstream behavior, prefer the smallest in-place shared-file diff with narrow markers. Do not move changed upstream logic into Cypher-owned code just to avoid textual conflicts, because that can create harder semantic merge conflicts.

## Core Rules

- Use `script/check-opencode-annotations.ts` as the source of truth for current shared scopes and exempt paths.
- Use `script/upstream/fix-cypher-markers.ts` for stale or broad markers, inspecting `--dry-run` output before applying changes.
- Treat upstream-owned files as shared unless the checker or repo ownership rules exempt them.
- Put Cypher-owned UI, CLI, runtime logic, and tests in Cypher-owned paths where practical.
- Avoid adding Cypher business logic directly to shared files.
- Keep shared-file edits as close as possible to upstream shape.
- Do not change shared files unless the change is required for Cypher functionality, fixes a Cypher bug, or is a minimal targeted upstream-quality fix.
- Do not create a large Cypher-only fork for a general upstream-quality improvement. Prefer a minimal targeted fix, or leave the broader change for upstream.
- Do not duplicate upstream logic unless there is a concrete reason. If duplication is unavoidable, isolate the Cypher delta and keep the upstream dependency obvious.

## Shared File Structure

- Do not refactor, rename, split files, or extract helpers in shared files just to improve readability or make Cypher extraction cleaner.
- Avoid structural changes that make upstream behavior harder to compare or hide semantic dependency on upstream code.

## Shared File Style

- Preserve upstream formatting and import style in shared files, even when it differs from Cypher style.
- Put Cypher-only imports on separate marked lines instead of reorganizing upstream imports.

## Decision Rules

Extract Cypher logic when:

- The change is an additive Cypher feature or integration, not a modification of existing upstream behavior.
- The shared-file change has meaningful Cypher-owned behavior, not just a tiny condition, import, registration, or field.
- The code has loops, branching, error handling, async workflows, storage access, network calls, UI rendering, or telemetry.
- The shared file can become a small orchestrator that calls Cypher helpers.
- The Cypher code is independent enough that extraction will not hide future upstream fixes or behavior changes.

Keep the change inline when:

- The Cypher delta is a single field, import, call, simple condition, or small registry entry.
- Extraction would reshape upstream code more than the Cypher change itself.
- The change modifies an upstream algorithm, ordering, heuristic, control flow, or bug fix.
- Extraction would duplicate upstream logic or hide semantic dependency on upstream behavior.
- The Cypher helper closes over upstream-local state. Keep closure-scoped helpers inline and contiguous in one narrow marker block.
- The shared file owns the only route table, enum, schema, switch, or registry where the hook must exist.
- The change restores upstream shape or removes a stale Cypher divergence.

Always preserve upstream behavior order unless the Cypher behavior change is intentional and tested.

## Marker Rules

- Mark only Cypher-specific diff lines in shared upstream files.
- Prefer inline markers for single-line changes: `const value = 42 // cypher_change`.
- Use block markers only for adjacent Cypher-specific lines:

```ts
// cypher_change start
registerCypherFeature(app)
// cypher_change end
```

- Use the file's native comment style, including JSX block comments inside JSX and `#` comments for YAML, TOML, and shell.
- Do not add markers in checker-exempt Cypher-owned paths.
- Remove stale markers when upstream already contains the behavior or when touching Cypher-owned files that still have old markers.
- Use `// cypher_change - new file` only for unavoidable new Cypher-specific files inside shared upstream paths.

## Tests

- Put Cypher-specific CLI/runtime tests in Cypher-owned test paths.
- Move tests out of shared upstream test paths when the behavior under test is Cypher-specific.
- Tests should cover the real failing path, not private or unstable APIs chosen only for convenience.
- Do not add skip gates for required regression coverage.

## Verification

After editing shared files or marker comments, run:

```bash
bun run script/check-opencode-annotations.ts
```

If the PR uses a non-default comparison base, pass the correct base ref:

```bash
bun run script/check-opencode-annotations.ts --base <base-ref>
```

For stale or broad markers in one shared file, inspect the dry run before applying:

```bash
bun run script/upstream/fix-cypher-markers.ts <repo-relative-file> --dry-run
```

Before finishing, confirm:

- Shared files contain minimal integration points only.
- Cypher logic and tests live in Cypher-owned paths where practical.
- Markers are narrow.
- Stale markers are removed.
- The annotation checker passed, or the reason it could not run is reported.
