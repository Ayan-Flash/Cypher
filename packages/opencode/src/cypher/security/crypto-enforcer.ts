import { Effect } from "effect";
import { SecurityError } from "./command-analyzer";

/**
 * Scan file content changes for weak cryptographic algorithms or hardcoded credentials.
 */
export function checkCryptoAndCredentials(content: string, filePath: string): Effect.Effect<void, SecurityError> {
  return Effect.gen(function* () {
    // Skip checking test files or markdown documents to prevent false positives in tests
    const isTestFile = /\b(?:test|spec|fixtures)\b/i.test(filePath);
    const isDocFile = filePath.endsWith(".md") || filePath.endsWith(".txt");
    if (isTestFile || isDocFile) return;

    // 1. Detect weak hashes (MD5 or SHA-1)
    if (
      /createHash\s*\(\s*['"](?:md5|sha1)['"]\s*\)/i.test(content) ||
      /\b(?:crypto\.subtle\.digest\s*\(\s*['"]sha-1['"])/i.test(content)
    ) {
      return yield* Effect.fail(
        new SecurityError(
          "Weak cryptographic hash algorithm (MD5 / SHA-1) detected in code modifications. Hashing password/data with weak algorithms is blocked."
        )
      );
    }

    // 2. Detect insecure symmetric cipher block modes (ECB / CBC without IV)
    if (
      /createCipheriv\s*\(\s*['"]aes-\d+-(?:ecb)['"]/i.test(content) ||
      /createCipher\s*\(\s*['"]aes-\d+-(?:cbc|ecb)['"]/i.test(content)
    ) {
      return yield* Effect.fail(
        new SecurityError(
          "Insecure cipher block mode (ECB / insecure CBC) detected in code modifications. Use secure authenticated modes like AES-GCM instead."
        )
      );
    }

    // 3. Detect hardcoded credentials / secret keys
    // Patterns matching api_key = "something", password = "value"
    const hardcodedSecretPattern = /(?:api[_-]?key|client[_-]?secret|aws[_-]?secret|private[_-]?key|db[_-]?password|auth[_-]?token)\s*[:=]\s*['"]([0-9a-zA-Z_\-]{16,})['"]/i;
    const match = content.match(hardcodedSecretPattern);
    if (match) {
      // Exclude obviously placeholder values
      const val = match[1].toLowerCase();
      const isPlaceholder = val.includes("test") || val.includes("fake") || val.includes("placeholder") || val.includes("example");
      if (!isPlaceholder) {
        return yield* Effect.fail(
          new SecurityError(
            `Hardcoded API Key, Token, or Client Secret candidate detected: "${match[0]}". Storing secrets in plaintext code is blocked.`
          )
        );
      }
    }
  });
}
