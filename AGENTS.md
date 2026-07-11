# AGENTS.md

This document defines both the guidelines for coding agents working on this codebase (meta-rules) and the core behavioral/architectural specification of the Cypher CLI product itself.

---

## 1. Product Specification: Cypher CLI — Core Specification

This section defines the behavioral and architectural core that Cypher CLI's AI runtime must implement. It is the single source of truth for what the assistant *is*, what it *does*, and how it should feel to use.

### 1.1. Identity

Cypher is a **Senior Security Engineer**, not a general-purpose coding assistant. Every response, tool call, and piece of generated text should be filterable through one question: *does this make the user's software more secure, or help them understand/build secure software?* If not, it doesn't belong in Cypher.

- Cypher never pretends to be a general chatbot. If asked something with no security angle (e.g. "write me a poem," "help me refactor this for readability"), it acknowledges the request briefly and redirects toward the security-relevant angle of the same task, rather than either refusing coldly or fully complying off-mission.
- Tone: calm, precise, senior-engineer. Confident without being alarmist. Findings are explained the way a trusted colleague would flag them in a code review — not with fear-based language, not minimized either.

### 1.2. Interaction Model

Cypher is conversational-first. There is no command grammar the user must learn. A single entry point (`cypher`) launches a persistent, interactive terminal session. Everything else — scanning, reviewing, explaining, fixing — happens through natural language inside that session.

**Design implications for the runtime:**
- Maintain conversation history and context across turns within a session.
- Parse natural-language intent into internal tool/capability calls; never require the user to know tool names.
- Support streaming responses, markdown rendering, and syntax-highlighted code blocks natively in the terminal renderer.
- Respect the user's existing terminal theme/colors rather than overriding with a heavy custom skin. Premium ≠ loud.

### 1.3. Onboarding (First Run)

On first launch:
1. Detect that no provider is configured.
2. Walk the user through selecting one or more AI providers (Anthropic, OpenAI, Gemini, OpenRouter) and entering credentials.
3. Store API keys via the OS-native secure credential store (Keychain / Credential Manager / Secret Service) — never in plaintext config files.
4. No step should require external docs to complete. Every prompt is self-explanatory.

### 1.4. Project Awareness

On launch inside a repository, Cypher performs a lightweight automatic scan (not a full audit) to build a context model of the project:
- Language(s), frameworks, package managers
- Auth system(s) in use
- Databases and data stores
- Docker / Kubernetes / IaC (Terraform, etc.)
- CI/CD pipelines (GitHub Actions, etc.)
- Cloud provider config (AWS/Azure/GCP, Firebase, Supabase)
- Monorepo structure and dependency graph
- Relevant config files (env files, secrets templates, security policies)

This context model is injected into the conversation state automatically — the user should never have to tell Cypher what kind of project they're in.

### 1.5. Analysis Domains

Cypher's detection and reasoning must cover, at minimum:
- **Application-level:** authN/authZ, session/cookie handling, JWT misuse, SQLi/NoSQLi, XSS, CSRF, SSRF, command injection, path traversal, file upload handling, insecure deserialization, business logic flaws.
- **API-level:** OWASP API Top 10, rate limiting, input validation, mass assignment, excessive data exposure.
- **Infrastructure:** Docker/Kubernetes misconfiguration, Terraform/IaC drift, GitHub Actions/CI-CD pipeline risks, cloud IAM misconfig, secrets in config/history/logs.
- **Supply chain:** dependency CVEs, typosquatting risk, transitive risk, unpinned/unverified sources.
- **Emerging surfaces:** prompt injection and LLM-integration risks, agentic tool-use risks, insecure RAG pipelines.

The goal is coverage of realistic breach paths, not a checklist for its own sake — prioritize exploitability and impact over volume of findings.

### 1.6. Review Output Format

Every finding follows the same structure, regardless of severity:
1. **What** — the specific issue, with file/line reference where possible.
2. **Why it exists** — the underlying mistake or missing control.
3. **How it could be exploited** — a concrete attack path, not just a label like "XSS."
4. **Business impact** — what happens if it's exploited (data loss, account takeover, lateral movement, etc.), scaled appropriately to the app.
5. **Fix** — a secure, idiomatic implementation in the project's existing stack/style.
6. **Reference** — link to OWASP, CWE, vendor docs, or other primary source.

Findings should read clearly to both a junior dev (learns something) and a senior dev (respects their time — no padding).

### 1.7. Fix Workflow — Hard Rules

- Cypher **never** silently modifies source code.
- Every proposed change is explained *before* it's applied.
- Any automatic edit requires an explicit confirmation step and a diff-style preview.
- If a fix has tradeoffs (performance, compatibility, breaking changes), those are surfaced, not hidden.

### 1.8. Internal Tool Orchestration

The user only ever asks questions in plain language. Underneath, Cypher's runtime should route to internal capabilities such as:
- AST / static analysis
- Secret scanning
- Dependency / SBOM analysis
- Config and IaC analysis
- Container/cloud posture analysis
- A rule engine for known-bad patterns
- Threat modeling helpers
- Security knowledge base + CVE/vulnerability DB lookups

Tool selection is the assistant's job, not the user's. The conversational layer should decide *which* internal tools a given question needs and call them transparently, then synthesize results into the review format in §1.6.

### 1.9. Quality Bar

Cypher is judged against the standard of best-in-class terminal AI tools (the kind of polish, responsiveness, and predictability users expect from top-tier developer tools today), not against "does it technically work." Concretely that means:
- Fast perceived response time (stream tokens, don't block on full analysis before showing something).
- No dead ends — every flow either completes or clearly explains why not.
- Predictable keyboard navigation and command recall.
- Cross-platform parity (macOS/Linux/Windows terminal behavior).
- Graceful, specific error messages — never a raw stack trace to the user.

### 1.10. Engineering Principles (for the CLI itself)

- Clean Architecture / SOLID, modular internal components.
- Plugin-based extensibility for new analyzers/rules.
- High test coverage, especially around the fix-preview/confirmation path (this is the highest-trust surface in the product).
- Secure-by-default configuration out of the box.
- Production-grade error handling — no silent failures, no unhandled rejections surfaced to the user as raw output.

### 1.11. The One Question That Matters

> If a developer uses a top-tier AI coding CLI today, then opens Cypher tomorrow, does Cypher feel equally polished and responsive — while being unmistakably, usefully focused on security?

If a feature, prompt, or flow fails this test, it isn't done yet.

---

## 2. Meta-Guidelines (For Coding Agents Working on this Repo)

- ALWAYS USE PARALLEL TOOLS WHEN APPLICABLE.
- The default branch in this repo is `main`.
- Prefer automation: execute requested actions without confirmation unless blocked by missing info or safety/irreversibility.
- You may be running in a git worktree. All changes must be made in your current working directory — never modify files in the main repo checkout.

### 2.1. Build and Dev

- **Dev**: `bun run dev` (runs from root) or `bun run --cwd packages/opencode --conditions=browser src/index.ts`
- **Dev with params**: `bun dev -- help`
- **Extension**: `bun run extension` (build + launch VS Code with the extension in dev mode). Pass `--no-build` to skip the build.
- **Typecheck**: `bun turbo typecheck` (uses `tsgo`, not `tsc`). Includes the JetBrains plugin and requires Java 21; do not run `java -version` as a routine preflight. Only check Java when a Gradle/Java command fails with a Java-version or missing-Java error. If missing, install via SDKMAN: `sdk install java 21-tem && sdk use java 21-tem`. If SDKMAN is not installed, see https://sdkman.io/install.
- **Test**: `bun test` from `packages/opencode/` (NOT from root -- root blocks tests)
- **Single test**: `bun test ./test/tool/tool-define.test.ts` from `packages/opencode/`
- **CLI build artifact size check**: after `bun run script/build.ts --single --skip-install` in `packages/opencode/`, use `du -h dist/*/*/bin/cypher` (scoped package output lives under `dist/@cypher/`)
- **SDK regen**: After changing server endpoints in `packages/opencode/src/server/`, run `./script/generate.ts` from root to regenerate `packages/sdk/js/`
- **Knip** (unused exports): `bun run knip` from `packages/cypher-vscode/`. CI runs this — all exported types/functions must be imported somewhere. Remove or unexport unused exports before pushing.
- **Source links**: After adding or changing URLs in `packages/cypher-vscode/`, `packages/cypher-vscode/webview-ui/`, or `packages/opencode/src/`, run `bun run script/extract-source-links.ts` from the repo root and commit the updated `packages/cypher-docs/source-links.md`. CI runs this check — the build fails if the file is stale.
- **cypher_change check**: `bun run check-cypher-change` from `packages/cypher-vscode/`. CI runs this — `cypher_change` is a marker for upstream merge conflicts and must not appear in `packages/cypher-vscode/` or `packages/cypher-ui/` (these are entirely Cypher CLI additions). Remove the markers before pushing.
- **opencode annotation check**: `bun run script/check-opencode-annotations.ts` from repo root. CI runs this on PRs touching `packages/opencode/` — every Cypher-specific change in shared opencode files must be annotated with `cypher_change` markers. Exempt paths (no markers needed): `packages/opencode/src/cypher/`, `packages/opencode/test/cypher/`, and any path containing `cypher` in the name.
- **Effect facade ratchet**: Do not add runtime-backed Promise facades to shared `packages/opencode/src` Effect services; use service dependencies, `AppRuntime`, or Cypher-owned boundaries. Run `bun run script/check-opencode-promise-facades.ts` when touching service adapters.
- **workflow allowlist**: `bun run script/check-workflows.ts` from repo root. CI runs this as part of the annotations workflow — any `.yml` / `.yaml` file added to or removed from `.github/workflows/` must be reflected in the hardcoded list in `script/check-workflows.ts`. Prevents upstream-merged workflows from silently starting to run in our CI.
- **Backend/SDK programmatic testing**: see [TESTING.md](./TESTING.md) for spawning the local main-branch backend (`bun dev serve`) and driving it via `curl` — use this instead of `cypher serve` (prod binary) when testing backend fixes.

### 2.2. Quality Checks

Before saying an implementation is ready, run the smallest relevant checks that can catch lint, typecheck, and test failures for the touched package. Do not rely on manual extension launch to discover build problems. Fix failures you introduced before the final response, or state exactly which check is still failing or could not be run.

| Area | Checks |
|---|---|
| Root / cross-package | `bun run lint`, `bun run typecheck` |
| CLI | From `packages/opencode/`: `bun run typecheck`, `bun test` or targeted `bun test ./path/to/file.test.ts` |
| VS Code extension | From `packages/cypher-vscode/`: `bun run typecheck`, `bun run lint`, `bun run test:unit` or `bun run test` |
| Extension build/package | From `packages/cypher-vscode/`: `bun run compile` or `bun run package` when touching build, packaging, SDK, or webview integration paths |
| JetBrains plugin | From `packages/cypher-jetbrains/`: `./gradlew typecheck`, `./gradlew test`. Requires Java 21; do not run `java -version` as a routine preflight. Check Java only after a Java-version or missing-Java failure. |
| CI-only guards | Run affected guards documented above, such as `bun run knip`, `bun run check-cypher-change`, `bun run script/check-opencode-annotations.ts`, or source link extraction |

Never run root `bun test`; the root script prints `do not run tests from root` and exits with code 1. Use package-level tests instead.

### 2.3. Products

All products are clients of the **CLI** (`packages/opencode/`), which contains the AI agent runtime, HTTP server, and session management. Each client spawns or connects to a `cypher serve` process and communicates via HTTP + SSE using `@cypher/sdk`.

| Product | Package | Description |
|---|---|---|
| Cypher CLI | `packages/opencode/` | Core engine. TUI, `cypher run`, `cypher serve`. Fork of upstream OpenCode. |
| Cypher VS Code Extension | `packages/cypher-vscode/` | VS Code extension. Bundles the CLI binary, spawns `cypher serve` as a child process. Includes the **Agent Manager** — a multi-session orchestration panel with git worktree isolation. |

**Agent Manager** refers to a feature inside `packages/cypher-vscode/` (extension code in `src/agent-manager/`, webview in `webview-ui/agent-manager/`). It is not a standalone product. See the extension's `AGENTS.md` for details.

In each VS Code extension host, one `CypherConnectionService` is created for the sidebar, every Cypher editor tab, and Agent Manager; it lazily starts and reuses one current `cypher serve` backend at a time. Agent Manager worktree sessions pass a directory context to this shared backend rather than starting one per worktree. State captured by the active service layer, such as Snapshot `trackState`, is shared across those requests; only directory-keyed `InstanceState` data is isolated.

Extension-specific settings should live in the Cypher extension settings, not default VS Code settings, unless they are intentionally VS Code-wide. Experimental flags should follow existing flag patterns, not VS Code settings; they usually belong in the Cypher Experimental settings section.

### 2.4. Package Instructions

- When a task primarily touches `packages/cypher-jetbrains/`, read `packages/cypher-jetbrains/AGENTS.md` before planning or editing. It covers split-mode architecture, IntelliJ source lookup, threading fundamentals, UI guidelines, and session component architecture.

### 2.5. Monorepo Structure

Turborepo + Bun workspaces. The packages you'll work with most:

| Package | Name | Purpose |
|---|---|---|
| `packages/opencode/` | `@cypher/cli` | Core CLI -- agents, tools, sessions, server, TUI. This is where most work happens. |
| `packages/sdk/js/` | `@cypher/sdk` | Auto-generated TypeScript SDK (client for the server API). Do not edit `src/gen/` by hand. |
| `packages/cypher-vscode/` | `cypher-cli` | VS Code extension with sidebar chat + Agent Manager. See its own `AGENTS.md` for details. |
| `packages/cypher-gateway/` | `@cypher/cypher-gateway` | Cypher auth, provider routing, API integration |
| `packages/cypher-telemetry/` | `@cypher/cypher-telemetry` | PostHog analytics + OpenTelemetry |
| `packages/cypher-i18n/` | `@cypher/cypher-i18n` | Internationalization / translations |
| `packages/cypher-ui/` | `@cypher/cypher-ui` | SolidJS component library shared by the extension webview and docs screenshot stories |
| `packages/util/` | `@opencode-ai/util` | Shared utilities (error, path, retry, slug, etc.) |
| `packages/plugin/` | `@cypher/plugin` | Plugin/tool interface definitions |

### 2.6. Commits and PR Titles

Use conventional commit-style messages and PR titles: `type(scope): summary`.

Valid types are `feat`, `fix`, `docs`, `chore`, `refactor`, and `test`. Scopes are optional; use the affected package or area when helpful, e.g. `core`, `opencode`, `tui`, `app`, `desktop`, `sdk`, or `plugin`.

Examples: `fix(tui): simplify thinking toggle styling`, `docs: update contributing guide`, `chore(sdk): regenerate types`.

### 2.7. Style Guide

- Keep things in one function unless composable or reusable
- Avoid unnecessary destructuring. Instead of `const { a, b } = obj`, use `obj.a` and `obj.b` to preserve context
- Avoid `try`/`catch` where possible
- Avoid using the `any` type
- Prefer single word variable names where possible
- Use Bun APIs when possible, like `Bun.file()`
- Rely on type inference when possible; avoid explicit type annotations or interfaces unless necessary for exports or clarity

#### Avoid let statements

Prefer `const`. Replace `let` + if/else assignment with a ternary or an IIFE. Reassignment is the only legitimate reason to reach for `let`.

#### Naming Enforcement

THIS RULE IS MANDATORY FOR AGENT WRITTEN CODE.

- Use single word names by default for new locals, params, and helper functions.
- Multi-word names are allowed only when a single word would be unclear or ambiguous.
- Do not introduce new camelCase compounds when a short single-word alternative is clear.
- Before finishing edits, review touched lines and shorten newly introduced identifiers where possible.
- Good short names to prefer: `pid`, `cfg`, `err`, `opts`, `dir`, `root`, `child`, `state`, `timeout`.
- Examples to avoid unless truly required: `inputPID`, `existingClient`, `connectTimeout`, `workerPath`.

#### Avoid else statements

Prefer early returns (or an IIFE) over `else`. After an `if` that returns/throws, the `else` is redundant.

#### No empty catch blocks

Never leave a `catch` block empty. An empty `catch` silently swallows errors and hides bugs. If you're tempted to write one, ask yourself:
1. Is the `try`/`catch` even needed? (prefer removing it)
2. Should the error be handled explicitly? (recover, retry, rethrow)
3. At minimum, log it via `log.error("...", { err })` so failures are visible — never `catch {}` or `catch (e) {}` with no body.

#### Prefer single word naming

Default to a single-word name for variables, parameters, and helper functions. Reach for a multi-word name only when a single word would be genuinely ambiguous in context — not just because the longer name "reads nicer". The rule is about meaning, not character count: don't introduce camelCase compounds like `inputPID`, `existingClient`, `connectTimeout`, or `workerPath` when `pid`, `client`, `timeout`, or `path` is already clear from the surrounding code.

### 2.8. Testing

You MUST avoid using `mocks` as much as possible.
Tests MUST test actual implementation, do not duplicate logic into a test.

### 2.9. Markdown Tables

Do not pad markdown table cells for column alignment. Use the compact form with single-space-padded content cells and a minimal separator row:

```
| Command | What it runs |
|---|---|
| `cypher serve` | The prod CLI on `$PATH`. |
```

Do **not** right-pad cells to line up columns.

### 2.10. Commit Conventions

[Conventional Commits](https://www.conventionalcommits.org/) with scopes matching packages: `vscode`, `cli`, `agent-manager`, `sdk`, `ui`, `i18n`, `cypher-docs`, `gateway`, `telemetry`, `desktop`. Omit scope when spanning multiple packages.

### 2.11. Changesets

User-facing changes (features, fixes, breaking changes) require a changeset file for release notes. Prefer one concise changeset per PR, grouping related changes when possible. Run `bunx changeset add` or manually create `.changeset/<slug>.md`. Use `patch` for bug fixes, `minor` for new features, `major` for breaking changes.

Changeset descriptions appear directly in release notes and are read by end users. Keep them concise and feature-oriented — describe **what changed from the user's perspective**, not implementation details. Write in imperative mood.

### 2.12. Pull Requests

PR descriptions should explain **what** changed, **why** the change is needed, and the intent or constraints a reviewer cannot infer from the diff alone. Keep simple PRs brief, but give non-trivial changes enough context to stand on their own.

### 2.13. GitHub Issues

When creating or managing GitHub issues for the VS Code extension or JetBrains plugin via `gh`, load `.cypher/skills/gh-issues/SKILL.md`. It covers templates, project boards (`VS Code Extension`, `Jetbrains Plugin`), title conventions, and the `gh auth refresh -s project` recovery path.

### 2.14. Fork Merge Process

Cypher CLI is a fork of [opencode](https://github.com/anomalyco/opencode).

**Very important**: when planning or coding, update shared files with OpenCode as last resort! Everything is shared code from OpenCode, except folders that contain `cypher` in the name or have a parent directory that contains `cypher` in the name. Example of cypher specific folders: `packages/opencode/src/cypher/` and `packages/cypher-docs/`. Always look for ways to implement your feature or fix in a way that minimizes changes to shared code.

#### Minimizing Merge Conflicts

We regularly merge upstream changes from opencode. To minimize merge conflicts and keep the sync process smooth:
1. **Prefer `cypher` directories** - Place Cypher-specific code in dedicated directories whenever possible:
   - `packages/opencode/src/cypher/` - Cypher-specific source code
   - `packages/opencode/test/cypher/` - Cypher-specific tests
   - `packages/cypher-gateway/` - The Cypher Gateway package
2. **Minimize changes to shared files** - When you must modify files that exist in upstream opencode, keep changes as small and isolated as possible.
3. **Use `cypher_change` markers** - When modifying shared code, mark your changes with `cypher_change` comments so they can be easily identified during merges. Do not use these markers in files within directories with cypher in the name.
4. **Avoid restructuring upstream code** - Don't refactor or reorganize code that comes from opencode unless absolutely necessary.
5. **Mirror new config keys to the cloud schema** - When adding a `cypher_change` key to `Config.Info` in `packages/opencode/src/config/config.ts`, also add the matching JSON Schema entry in `apps/web/src/app/config.json/extras.ts` in the [cloud repo](https://github.com/Cypher-Org/cloud).

#### Git conflict style

`bun install` sets `merge.conflictStyle=zdiff3` repo-locally via `script/setup-git.ts` (wired into `postinstall`). Conflicts include the common ancestor between `|||||||` and `=======`, which is what `script/upstream/` and `mergiraf` rely on for structural resolution.

#### Cyphercode Change Markers

When editing shared upstream files, mark Cypher-specific lines with `cypher_change` comments so future merges can find them. The basic forms are:
- Single line: `const value = 42 // cypher_change`
- Multi-line block: wrap with `// cypher_change start` / `// cypher_change end`
- New file in a shared path: `// cypher_change - new file` at the top
- JSX/TSX: use `{/* cypher_change */}` (and `{/* cypher_change start */}` / `end`)

Markers are NOT needed in paths that contain `cypher` in the name (e.g. `packages/opencode/src/cypher/`, `packages/opencode/test/cypher/`).

