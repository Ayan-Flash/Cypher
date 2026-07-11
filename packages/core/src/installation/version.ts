declare global {
  const CYPHER_VERSION: string
  const CYPHER_CHANNEL: string
  const CYPHER_BUILD_KIND: string // cypher_change
}

export const InstallationVersion = typeof CYPHER_VERSION === "string" ? CYPHER_VERSION : "local"
export const InstallationChannel = typeof CYPHER_CHANNEL === "string" ? CYPHER_CHANNEL : "local"
export const InstallationLocal = InstallationChannel === "local"
// cypher_change start - distinguish release builds from source / local builds
export const InstallationBuildKind: "source" | "release" =
  typeof CYPHER_BUILD_KIND === "string" && CYPHER_BUILD_KIND === "release" ? "release" : "source"
// cypher_change end
