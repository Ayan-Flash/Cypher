#!/usr/bin/env bun
// cypher_change - new file

/**
 * Guards generated Cypher config dependency artifacts.
 *
 * Cypher loads project config from .cypher/ and .cypher/ and installs
 * @cypher/plugin there at runtime. npm writes package.json, lockfiles,
 * .gitignore, and node_modules as generated local state. These paths must stay
 * untracked so background installs do not create recurring branch diffs.
 */

import { spawnSync } from "node:child_process"

const paths = [
  ".cypher/.gitignore",
  ".cypher/package.json",
  ".cypher/package-lock.json",
  ".cypher/pnpm-lock.yaml",
  ".cypher/bun.lock",
  ".cypher/yarn.lock",
  ".cypher/node_modules",
  ".cypher/.gitignore",
  ".cypher/package.json",
  ".cypher/package-lock.json",
  ".cypher/pnpm-lock.yaml",
  ".cypher/bun.lock",
  ".cypher/yarn.lock",
  ".cypher/node_modules",
]

const git = spawnSync("git", ["ls-files", "-z", "--", ...paths], { encoding: "utf8" })

if (git.status !== 0) {
  console.error(git.stderr.trim() || "git ls-files failed")
  process.exit(1)
}

const bad = git.stdout.split("\0").filter(Boolean).sort()

if (bad.length === 0) {
  console.log("check-cypher-generated-artifacts: ok")
  process.exit(0)
}

console.error("Generated Cypher config dependency artifacts are tracked:")
for (const file of bad) console.error(`  ${file}`)
console.error("")
console.error("These files are created by runtime dependency installs in .cypher/ and .cypher/.")
console.error("Remove them from git and keep them ignored.")
process.exit(1)
