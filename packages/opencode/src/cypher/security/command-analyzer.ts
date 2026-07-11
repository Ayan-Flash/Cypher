import { Effect } from "effect";

export class SecurityError extends Error {
  readonly _tag = "SecurityError";
  constructor(message: string) {
    super(`[Security Gating Exception] ${message}`);
    this.name = "SecurityError";
  }
}

/**
 * Scan a shell command for known injection, hijacking, or credential leak risks.
 */
export function analyzeCommand(command: string): Effect.Effect<void, SecurityError> {
  return Effect.gen(function* () {
    const trimmed = command.trim();
    if (!trimmed) return;

    // 1. Check for reverse shells
    if (
      trimmed.includes("/dev/tcp/") ||
      trimmed.includes("/dev/udp/") ||
      /\bnc\s+.*-[ec]\b/i.test(trimmed) ||
      /\bnetcat\s+.*-[ec]\b/i.test(trimmed) ||
      /\bbash\s+-i\b/i.test(trimmed)
    ) {
      return yield* Effect.fail(
        new SecurityError("Reverse shell signature or raw socket redirection detected. Command blocked.")
      );
    }

    // 2. Check for base64 / obfuscation execution piping
    if (
      /base64\s+(?:-d|--decode)\b/i.test(trimmed) &&
      /\|\s*(?:bash|sh|eval|iex|powershell)\b/i.test(trimmed)
    ) {
      return yield* Effect.fail(
        new SecurityError("Obfuscated base64 payload piped into an execution shell detected. Command blocked.")
      );
    }

    // 3. Check for curl/wget piped to bash/sh (untrusted remote script execution)
    if (
      /(?:curl|wget)\b/i.test(trimmed) &&
      /\|\s*(?:bash|sh|eval)\b/i.test(trimmed)
    ) {
      return yield* Effect.fail(
        new SecurityError("Untrusted remote script execution detected (curl/wget piped directly to shell). Command blocked.")
      );
    }

    // 4. Check for unauthorized access to sensitive credential / config directories
    const sensitivePattern = /\b(?:\.env|\.git\/config|id_rsa|id_dsa|id_ed25519|\.ssh\b|\.aws\b)\b/i;
    const destructiveAccess = /\b(?:rm|del|cp|copy|mv|move|cat|type|get-content|set-content|add-content|remove-item)\b/i;
    if (sensitivePattern.test(trimmed) && destructiveAccess.test(trimmed)) {
      return yield* Effect.fail(
        new SecurityError(
          "Command attempts to access or modify highly sensitive security files or credentials (.env, .git/config, .ssh, .aws). Access blocked."
        )
      );
    }

    // 5. Block resource-intensive recursive directory listing commands (e.g. ls -R, dir /s)
    if (
      /\bls\s+-(?:[a-z]*R[a-z]*)\b/i.test(trimmed) ||
      /\bdir\s+(?:.*\/s|.*\/S)\b/i.test(trimmed)
    ) {
      return yield* Effect.fail(
        new SecurityError(
          "Recursive directory listing commands (ls -R / dir /s) are blocked to prevent workspace hangs. Please use the native 'glob' or 'grep' tools instead."
        )
      );
    }
  });
}
