# Upstream Merge Automation

Scripts for automating the merge of upstream opencode changes into Cypher.

## Quick Start

```bash
# Install dependencies (from script/upstream directory)
cd script/upstream
bun install

# List available upstream versions
bun run list-versions.ts

# Analyze changes for a specific version (without merging)
bun run analyze.ts --version v1.1.49

# Run the full merge process
bun run merge.ts --version v1.1.49

# Dry-run to preview what would happen
bun run merge.ts --version v1.1.49 --dry-run

# Use a different base branch (e.g., for incremental merges)
bun run merge.ts --version v1.1.50 --base-branch catrielmuller/cypher-opencode-v1.1.44
```

## Scripts

### Main Scripts

| Script | Description |
|---|---|
| `merge.ts` | Main orchestration script for upstream merges |
| `list-versions.ts` | List available upstream versions |
| `analyze.ts` | Analyze changes without merging |
| `opencode-changesets.ts` | Generate Cypher changesets from upstream opencode release notes |
| `fix-cypher-markers.ts` | Rebuild `cypher_change` markers for one file against the last merged upstream |
| `reset-to-upstream.ts` | Reset one file to the transformed last merged upstream version |
| `find-reset-candidates.ts` | Bulk-find files that have drifted insignificantly from upstream and (optionally) reset them |

### Transform Scripts

| Script | Description |
|---|---|
| `transforms/package-names.ts` | Transform opencode package names to cypher |
| `transforms/preserve-versions.ts` | Preserve Cypher's package versions |
| `transforms/keep-ours.ts` | Keep Cypher's version of specific files |
| `transforms/skip-files.ts` | Skip/remove files that shouldn't exist in Cypher |
| `transforms/transform-i18n.ts` | Transform i18n files with Cypher branding |
| `transforms/transform-take-theirs.ts` | Take upstream + apply Cypher branding for branding-only files |
| `transforms/transform-package-json.ts` | Enhanced package.json with Cypher dependency injection and newest-Bun-wins reconciliation |
| `transforms/transform-scripts.ts` | Transform script files with GitHub API references |
| `transforms/transform-extensions.ts` | Transform extension files (Zed, etc.) |
| `transforms/transform-web.ts` | Transform web/docs files (.mdx) |

### Codemods (AST-based)

| Script | Description |
|---|---|
| `codemods/transform-imports.ts` | Transform import statements using ts-morph |
| `codemods/transform-strings.ts` | Transform string literals |

## Release Notes Changesets

After merging upstream opencode releases, use `opencode-changesets.ts` to turn the upstream GitHub release notes into Cypher changesets:

```bash
bun script/upstream/opencode-changesets.ts --from 1.17.0 --to 1.17.7
```

The script fetches releases from `anomalyco/opencode`, selects published releases in the semver range `(from, to]`, and writes one `.changeset/opencode-vX-Y-Z-to-vX-Y-Z.md` file for the whole range. It requires the target release to exist, merges notes from every release into shared `##` sections and `###` categories, then folds those headings into each bullet (for example, `Core Bugfixes: ...`) so Changesets can embed the notes cleanly in package changelogs. It generates a patch changeset for the fixed release group, `@cypher/cli` and `cypher-cli`. Generated notes omit contributor thank-you blocks and the upstream `Desktop` and `SDK` sections by default because Cypher does not ship the opencode desktop app and SDK release notes are not user-facing for Cypher.

## Merge Process

The merge automation follows this process, applying **all transformations BEFORE the merge** to minimize conflicts:

1. **Validate environment**
   - Check for upstream remote
   - Ensure working directory is clean

2. **Fetch upstream** and determine target version

3. **Generate conflict report** analyzing which files will conflict

4. **Create branches**
   - `backup/<branch>-<timestamp>` - Backup of current state
   - `<author>/cypher-opencode-<version>` - Merge target branch
   - `<author>/opencode-<version>` - Transformed upstream branch

5. **Apply ALL transformations to upstream branch (PRE-MERGE)**:
   - Remove files that should not exist in Cypher (`skipFiles`)
   - Transform package names (opencode-ai -> @cypher/cli)
   - Preserve Cypher's versions
   - Transform i18n files with Cypher branding
   - Transform branding-only files (UI components, configs)
   - Transform package.json files (names, deps, Cypher injections)
   - Transform script files (GitHub API references)
   - Transform extension files (Zed, etc.)
   - Transform web/docs files
   - Reset Cypher-specific files

6. **Merge** transformed upstream into Cypher branch
   - Since all branding transforms are applied pre-merge, conflicts should be minimal
   - Remaining conflicts are files with actual code differences (cypher_change markers)

7. **Auto-resolve** any remaining conflicts
   - Skip files that shouldn't exist in Cypher
   - Keep Cypher's version of specific files
   - Fallback transforms for edge cases

8. **Push** and generate final report

## Configuration

Configuration is defined in `utils/config.ts`:

```typescript
{
  // Package name mappings
  packageMappings: [
    { from: "opencode-ai", to: "@cypher/cli" },
    { from: "@opencode-ai/cli", to: "@cypher/cli" },
    // ...
  ],

  // Files to always keep Cypher's version (never take upstream)
  keepOurs: [
    "README.md",
    "CONTRIBUTING.md",
    "AGENTS.md",
    ".github/workflows/publish.yml",  // GitHub workflows - manual review
    // ...
  ],

  // Files to skip entirely (remove from merge)
  skipFiles: [
    "README.*.md",  // Translated READMEs
    "STATS.md",
    ".github/workflows/update-nix-hashes.yml",
    // ...
  ],

  // Files to take upstream + apply Cypher branding transforms
  takeTheirsAndTransform: [
    "packages/ui/src/**/*.tsx",
    // ...
  ],

  // Cypher-specific directories (preserved)
  cypherDirectories: [
    "packages/opencode/src/cypher",
    "packages/cypher-gateway",
    "packages/cypher-telemetry",
    // ...
  ],
}
```

## Pre-Merge Transformation Strategy

**Key insight**: By applying all branding transforms to the upstream branch BEFORE merging, we eliminate most conflicts that would otherwise occur due to branding differences (OpenCode -> Cypher).

### Transform Order (Pre-Merge)

The following transforms are applied to the opencode branch before merging:

1. **Skip files** - Remove upstream-only packages/files that should not exist in Cypher
2. **Package names** - `opencode-ai` -> `@cypher/cli`, etc.
3. **Versions** - Preserve Cypher's version numbers
4. **i18n files** - OpenCode -> Cypher in user-visible strings
5. **Branding files** - UI components, configs with branding only
6. **package.json** - Names, dependencies, Cypher injections
7. **Scripts** - GitHub API references
8. **Extensions** - Zed, etc.
9. **Web/docs** - Documentation files

### Post-Merge Strategies

After merging, any remaining conflicts are handled based on file type:

| File Type | Strategy | Description |
|---|---|---|
| i18n files | `i18n-transform` | Take upstream, apply Cypher branding |
| UI components | `take-theirs-transform` | Take upstream, apply branding (no logic changes) |
| package.json | `package-transform` | Take upstream, transform names, inject Cypher deps |
| Script files | `script-transform` | Take upstream, transform GitHub references |
| Extensions | `extension-transform` | Take upstream, apply branding |
| Web/docs | `web-transform` | Take upstream, apply branding |
| README/docs | `keep-ours` | Keep Cypher's version |
| GitHub workflows | `keep-ours` | Keep Cypher's version (manual review) |
| Code with markers | `manual` | Has `cypher_change` markers, needs review |

### Why This Reduces Conflicts

Previously, conflicts occurred because:

- Upstream had `OpenCode` branding
- Cypher had `Cypher` branding
- Git saw these as conflicting changes

Now:

- We transform upstream to `Cypher` branding BEFORE merge
- Both branches have the same branding
- Git sees no conflict for branding-only files

The only remaining conflicts are files with **actual code differences** - files with `cypher_change` markers that contain Cypher-specific logic.

### Bun Version Safety

Root `package.json` reconciliation uses the newer valid `packageManager` Bun version from Cypher and upstream. An older upstream version cannot downgrade Cypher, while a newer upstream version is retained as an upgrade. Before the merge is finalized, `merge.ts` also validates the result against the pristine Cypher base and upstream commit and aborts if the merged Bun version is lower than either input.

## CLI Options

### merge.ts

```
Options:
  --version <version>    Target upstream version (e.g., v1.1.49)
  --commit <hash>        Target upstream commit hash
  --base-branch <name>   Base branch to merge into; use HEAD for current branch (default: main)
  --dry-run              Preview changes without applying them
  --no-push              Don't push branches to remote
  --no-worktrees         Don't create reference worktrees
  --report-only          Only generate conflict report
  --verbose              Enable verbose logging
  --author <name>        Author name for branch prefix
```

By default, `merge.ts` also prepares prompt-friendly reference worktrees under `.worktrees/opencode-merge/`:

| Path | Snapshot |
|---|---|
| `.worktrees/opencode-merge/opencode` | Pristine upstream opencode at the requested version or commit |
| `.worktrees/opencode-merge/cypher-main` | The Cypher base branch snapshot used for the merge |
| `.worktrees/opencode-merge/auto-merge` | The automated merge result before final lockfile or SDK regeneration |

If conflicts remain after automation, `auto-merge` is a committed local snapshot branch that may intentionally contain conflict markers as normal file content. The real merge branch remains unresolved so manual resolution can continue with accurate git conflict state.

### analyze.ts

```
Options:
  --version <version>    Target upstream version
  --commit <hash>        Target commit hash
  --base-branch <name>   Base branch to analyze from (default: main)
  --output <file>        Output file for report
```

### fix-cypher-markers.ts

```
Usage:
  bun run script/upstream/fix-cypher-markers.ts <repo-relative-file> [--dry-run]

Options:
  --dry-run              Show what would change without writing the file
```

The command finds the newest upstream tag already merged into `HEAD` (read from `.opencode-version` at the repo root, falling back to an `ls-remote` + `merge-base --is-ancestor` walk), reads that upstream version of the file, applies the same branding transforms used by upstream merge automation, strips existing `cypher_change` markers from the current file, and adds fresh markers around the remaining lines that differ from upstream.

The `.opencode-version` file is a single-line tag (e.g. `v1.14.33`) recorded by `merge.ts` after every successful upstream merge. Editing it by hand pins the "last merged" tag for the per-file commands above; delete it to fall back to the slower automatic discovery.

### reset-to-upstream.ts

```
Usage:
  bun run script/upstream/reset-to-upstream.ts <repo-relative-file> [--dry-run]

Options:
  --dry-run              Show what would change without writing the file
```

The command finds the newest upstream tag already merged into `HEAD`, reads that upstream version of the file, applies the same branding transforms used by upstream merge automation for text files, and writes the result to the working tree. Binary files are restored as raw upstream bytes without text transforms. If the file does not exist upstream, the local file is deleted.

### find-reset-candidates.ts

```
Usage:
  bun run script/upstream/find-reset-candidates.ts [path] [options]

Arguments:
  path                     Optional repo-relative subdirectory to scope to.
                           Defaults to all tracked shared paths.

Options:
  --review-limit <n>       Max non-marker, non-whitespace diff lines that
                           still auto-resets (default: 5).
  --dry-run                Classify and report only; do not write any files.
  --concurrency <n>        Parallel classifications (default: 8).
```

The command pre-filters with `git diff --name-only <last-merged-upstream>..HEAD` and drops:

- Cypher-only paths: anything under `packages/cypher-*/`, any `**/cypher/**` subdir, `script/upstream/`.
- Non-code assets: SVG, PNG, fonts, archives, lock files, etc. (see `SKIP_EXTENSIONS` / `SKIP_FILENAMES` in the script).
- Files covered by the merge config's `keepOurs` or `skipFiles` lists in `utils/config.ts` — these are intentionally preserved or removed in Cypher and must not be bulk-reset.

It then issues one `git cat-file --batch-check` for all remaining paths to grab upstream blob sizes in a single subprocess. Files absent upstream land in `upstream-missing` immediately; files above 256 KB land in `too-large` (generated manifests, giant snapshots). Only the survivors get fetched via `git show` and classified:

| Bucket | Meaning | Action |
|---|---|---|
| `identical` | Local bytes already match transformed upstream (branding-only drift in raw git diff) | none |
| `markers-only` | Stripping `cypher_change` markers makes local match upstream | reset |
| `cosmetic-only` | Non-marker diff is only whitespace or reordered lines (the line multiset is identical) | reset |
| `small-diff` | ≤ `--review-limit` non-marker, non-cosmetic diff lines | reset |
| `large-diff` | > `--review-limit` non-marker, non-cosmetic diff lines | skipped |
| `upstream-missing` | File does not exist upstream (cypher-only, intentional) | skipped |
| `local-missing` | File tracked but missing locally (deleted in Cypher) | skipped |
| `binary-diff` | Binary file differs | skipped (use `reset-to-upstream.ts` per file) |
| `binary-identical` | Binary file already matches | none |
| `too-large` | Upstream blob > 256 KB | skipped (use `reset-to-upstream.ts` per file) |

Line counting uses an in-process multiset diff (pure JS, no subprocess) for speed and robustness against concurrent git output stalls on big files. Moved/reordered lines therefore count as zero drift, which is usually what you want for "is this file meaningfully different from upstream".

`markers-only`, `cosmetic-only`, and `small-diff` buckets are auto-reset unless `--dry-run` is passed. A markdown summary is printed to stdout so you can review what happened and spot-check the resulting `git diff`. All resets land as uncommitted working-tree changes; `git diff` / `git checkout` is your safety net.

Tighten the blast radius with `--review-limit 0` (only `markers-only` and `cosmetic-only`) or by scoping with a `path` argument (e.g. `packages/opencode/src/mcp`).

## Using Custom Base Branches

By default, upstream merges start from the `main` branch. However, you can use `--base-branch` to start from a different branch. This is useful for:

Passing `--base-branch HEAD` targets the currently checked-out branch without typing its full name.

### Incremental Merges

When working on multiple upstream versions, you can create a chain of merge PRs:

```bash
# First merge: v1.1.44 into main
bun run merge.ts --version v1.1.44

# Create PR: catrielmuller/cypher-opencode-v1.1.44 -> main

# Second merge: v1.1.50 based on the previous PR (without waiting for approval)
bun run merge.ts --version v1.1.50 --base-branch catrielmuller/cypher-opencode-v1.1.44

# Create PR: catrielmuller/cypher-opencode-v1.1.50 -> catrielmuller/cypher-opencode-v1.1.44
# OR: catrielmuller/cypher-opencode-v1.1.50 -> main (once first PR is merged)
```

### Benefits

- **Work in parallel**: Don't wait for PR approval to start the next merge
- **Isolation**: Each merge is independent and easier to review
- **Flexibility**: Can adjust the PR chain as needed
- **Cleaner history**: Related merges can be grouped together

### Example Workflow

```bash
# 1. Analyze next version from your WIP branch
bun run analyze.ts --version v1.1.50 --base-branch catrielmuller/cypher-opencode-v1.1.44

# 2. Run the merge
bun run merge.ts --version v1.1.50 --base-branch catrielmuller/cypher-opencode-v1.1.44

# 3. Create PR from catrielmuller/cypher-opencode-v1.1.50
#    - Target: catrielmuller/cypher-opencode-v1.1.44 (if first PR not merged yet)
#    - Target: main (if first PR is already merged)
```

## Manual Conflict Resolution

After running the merge script, you may have remaining conflicts. To resolve:

1. Open each conflicted file
2. Look for `cypher_change` markers to identify Cypher-specific code
3. Resolve conflicts, keeping Cypher-specific changes
4. Stage and commit:
   ```bash
   git add -A
   git commit -m "resolve merge conflicts"
   ```

## Rollback

If something goes wrong:

```bash
# Find your backup branch
git branch | grep backup

# Reset to backup
git checkout main
git reset --hard backup/main-<timestamp>
```

## Adding New Transformations

### String-based (simple)

Edit `transforms/package-names.ts` and add patterns to `PACKAGE_PATTERNS`.

### AST-based (robust)

1. Create a new file in `codemods/`
2. Use ts-morph for TypeScript AST manipulation
3. Export transform functions
4. Add to the merge orchestration if needed

## Troubleshooting

### "No upstream remote found"

```bash
git remote add upstream git@github.com:anomalyco/opencode.git
```

### "Working directory has uncommitted changes"

```bash
git stash
# or
git commit -am "WIP"
```

### Merge conflicts after auto-resolution

Some files require manual review. Check the generated report for guidance.
