import { Effect } from "effect";
import { SecurityError } from "./command-analyzer";

// Levenshtein distance for typosquatting check
function getLevenshteinDistance(a: string, b: string): number {
  const tmp = [];
  let i, j;
  for (i = 0; i <= a.length; i++) {
    tmp.push([i]);
  }
  for (j = 0; j <= b.length; j++) {
    tmp[0][j] = j;
  }
  for (i = 1; i <= a.length; i++) {
    for (j = 1; j <= b.length; j++) {
      tmp[i][j] = Math.min(
        tmp[i - 1][j] + 1,
        tmp[i][j - 1] + 1,
        tmp[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1)
      );
    }
  }
  return tmp[a.length][b.length];
}

const POPULAR_PACKAGES = [
  "express", "lodash", "react", "vue", "angular", "typescript",
  "requests", "urllib3", "numpy", "pandas", "flask", "django",
  "chalk", "commander", "axios", "uuid", "dotenv", "tslib",
  "fs-extra", "glob", "inquirer", "semver", "minimist"
];

const DEPRECATED_OR_UNSAFE = new Set([
  "request", // Deprecated since 2020, use axios/fetch
  "node-uuid", // Obsolete/insecure, use uuid
  "event-stream", // Historically compromised package
  "flatmap-stream", // Historically compromised package
  "pepy", // Compromised/malicious package
]);

/**
 * Scan installation commands for typosquatting, deprecated libraries, or missing version pins.
 */
export function analyzeDependencies(command: string): Effect.Effect<void, SecurityError> {
  return Effect.gen(function* () {
    const trimmed = command.trim();
    if (!trimmed) return;

    // Detect package install commands
    const isNPMInstall = /\b(?:npm\s+i(?:nstall)?|yarn\s+add|pnpm\s+add|bun\s+add|pip\s+install)\b/i.test(trimmed);
    if (!isNPMInstall) return;

    // Extract potential package names
    // Examples: npm i express lodash or pip install requests
    const tokens = trimmed.split(/\s+/);
    const installIndex = tokens.findIndex(t => /^(?:i|install|add)$/i.test(t));
    if (installIndex === -1 || installIndex === tokens.length - 1) return;

    const packages = tokens.slice(installIndex + 1).filter(t => !t.startsWith("-"));

    for (const pkg of packages) {
      // Split name and version (e.g. express@4.18.2 or requests==2.28.1)
      const namePart = pkg.split(/[@=]/)[0].trim().toLowerCase();
      if (!namePart) continue;

      // 1. Check for deprecated or unsafe packages
      if (DEPRECATED_OR_UNSAFE.has(namePart)) {
        return yield* Effect.fail(
          new SecurityError(`Package "${namePart}" is deprecated, insecure, or historically compromised. Installation blocked.`)
        );
      }

      // 2. Typosquatting checks
      for (const popular of POPULAR_PACKAGES) {
        if (namePart === popular) continue;
        const distance = getLevenshteinDistance(namePart, popular);
        if (distance === 1 || (distance === 2 && namePart.length > 4)) {
          return yield* Effect.fail(
            new SecurityError(`Potential typosquatting attempt detected: "${namePart}" is very similar to the popular package "${popular}". Installation blocked.`)
          );
        }
      }

      // 3. Version pinning checker
      // Encourages pinned dependencies in installation commands
      const hasVersionPin = pkg.includes("@") || pkg.includes("==") || pkg.includes(">=");
      if (!hasVersionPin) {
        // Just return a warning message in logs/stderr (non-blocking, but encourages best practices)
        console.warn(`[Security Warning] Package "${pkg}" is not pinned to a specific version. We recommend using version constraints.`);
      }
    }
  });
}
