import { $ } from "bun"
import semver from "semver"
import path from "path"

const rootPkgPath = path.resolve(import.meta.dir, "../../../package.json")
const rootPkg = await Bun.file(rootPkgPath).json()
const expectedBunVersion = rootPkg.packageManager?.split("@")[1]

if (!expectedBunVersion) {
  throw new Error("packageManager field not found in root package.json")
}

// relax version requirement
const expectedBunVersionRange = `^${expectedBunVersion}`

if (!semver.satisfies(process.versions.bun, expectedBunVersionRange)) {
  throw new Error(`This script requires bun@${expectedBunVersionRange}, but you are using bun@${process.versions.bun}`)
}
// cypher_change start
const env = {
  CYPHER_CHANNEL: process.env["CYPHER_CHANNEL"],
  CYPHER_BUMP: process.env["CYPHER_BUMP"],
  CYPHER_VERSION: process.env["CYPHER_VERSION"],
  CYPHER_RELEASE: process.env["CYPHER_RELEASE"],
  CYPHER_PRE_RELEASE: process.env["CYPHER_PRE_RELEASE"],
}
// cypher_change end
const CHANNEL = await (async () => {
  if (env.CYPHER_CHANNEL) return env.CYPHER_CHANNEL // cypher_change
  // cypher_change start - publish to "rc" channel for pre-releases
  if (env.CYPHER_PRE_RELEASE === "true") return "rc"
  // cypher_change end
  if (env.CYPHER_BUMP) return "latest" // cypher_change
  if (env.CYPHER_VERSION && !env.CYPHER_VERSION.startsWith("0.0.0-")) return "latest" // cypher_change
  return await $`git branch --show-current`.text().then((x) => x.trim().replace(/[^0-9A-Za-z-]/g, "-")) // cypher_change
})()
const IS_PREVIEW = CHANNEL !== "latest"

// cypher_change start - shared helpers for version computation
function parseVersion(input: string) {
  const match = input.trim().match(/^v?(\d+)\.(\d+)\.(\d+)$/)
  if (!match) return
  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3]),
    value: `${match[1]}.${match[2]}.${match[3]}`,
  }
}

function compareVersion(
  a: NonNullable<ReturnType<typeof parseVersion>>,
  b: NonNullable<ReturnType<typeof parseVersion>>,
) {
  if (a.major !== b.major) return a.major - b.major
  if (a.minor !== b.minor) return a.minor - b.minor
  return a.patch - b.patch
}

async function fetchLatest() {
  const data: any = await fetch("https://registry.npmjs.org/@cypher/cli/latest").then((res) => {
    if (!res.ok) throw new Error(res.statusText)
    return res.json()
  })
  return data.version as string
}

async function fetchHighest() {
  if (!process.env.GH_REPO) return fetchLatest()
  const data: { tagName: string }[] = await $`gh release list --json tagName --limit 100 --repo ${process.env.GH_REPO}`
    .json()
    .catch(() => [])
  const versions = data.flatMap((item) => {
    const version = parseVersion(item.tagName)
    if (!version) return []
    return [version]
  })
  const highest = versions.sort(compareVersion).at(-1)
  if (highest) return highest.value
  return fetchLatest()
}

function bumpVersion(current: string, type: string) {
  const version = parseVersion(current)
  if (!version) throw new Error(`Invalid version: ${current}`)
  if (type === "major") return `${version.major + 1}.0.0`
  if (type === "minor") return `${version.major}.${version.minor + 1}.0`
  return `${version.major}.${version.minor}.${version.patch + 1}`
}
// cypher_change end

const VERSION = await (async () => {
  if (env.CYPHER_VERSION) return env.CYPHER_VERSION
  if (IS_PREVIEW) {
    // cypher_change start - rc releases use plain semver required by VS Code Marketplace
    if (env.CYPHER_BUMP && env.CYPHER_PRE_RELEASE === "true") {
      const current = await fetchHighest()
      return bumpVersion(current, env.CYPHER_BUMP.toLowerCase())
    }
    // cypher_change end
    return `0.0.0-${CHANNEL}-${new Date().toISOString().slice(0, 16).replace(/[-:T]/g, "")}`
  }
  const version = await fetchHighest() // cypher_change
  return bumpVersion(version, env.CYPHER_BUMP?.toLowerCase() ?? "patch") // cypher_change
})()

// cypher_change start
const team = [
  "actions-user",
  "cypher-maintainer[bot]",
  "cypherconnect[bot]",
  "cypherconnect-lite[bot]",
  "alexkgold",
  "arimesser",
  "arkadiykondrashov",
  "bturcotte520",
  "catrielmuller",
  "chrarnoldus",
  "codingelves",
  "darkogj",
  "dependabot[bot]",
  "dosire",
  "DScdng",
  "emilieschario",
  "eshurakov",
  "Helix-Cypher",
  "iscekic",
  "jeanduplessis",
  "jobrietbergen",
  "jrf0110",
  "johnnyeric",
  "alex-alecu",
  "imanolmzd-svg",
  "cypher-bot",
  "cypher-cli-bot",
  "cypher-cli-bot[bot]",
  "kirillk",
  "lambertjosh",
  "LigiaZ",
  "marius-cypher",
  "markijbema",
  "olearycrew",
  "pandemicsyn",
  "pedroheyerdahl",
  "RSO",
  "sbreitenother",
  "suhailkc2025",
  "Sureshkumars",
]
// cypher_change end

export const Script = {
  get channel() {
    return CHANNEL
  },
  get version() {
    return VERSION
  },
  get preview() {
    return IS_PREVIEW
  },
  get release(): boolean {
    return !!env.CYPHER_RELEASE
  },
  get team() {
    return team
  },
}
console.log(`cypher script`, JSON.stringify(Script, null, 2)) // cypher_change
