#!/usr/bin/env bun

import { $ } from "bun"
import fs from "fs"
import os from "os" // cypher_change
import path from "path"
import { fileURLToPath } from "url"
import { createSolidTransformPlugin } from "@opentui/solid/bun-plugin"
import { createRequire } from "module" // cypher_change

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const dir = path.resolve(__dirname, "..")
const require = createRequire(import.meta.url) // cypher_change

process.chdir(dir)

const generated = await import("./generate")

import { Script } from "@opencode-ai/script"
import pkg from "../package.json"
// cypher_change start
import { stageBubblewrap } from "./cypher/bubblewrap"
import { LanceDBRuntime } from "../src/cypher/lancedb"
import { CypherSandboxWorker } from "./cypher/cypher-sandbox-worker"
// cypher_change end

// Load migrations from migration directories
const migrationDirs = (
  await fs.promises.readdir(path.join(dir, "migration"), {
    withFileTypes: true,
  })
)
  .filter((entry) => entry.isDirectory() && /^\d{4}\d{2}\d{2}\d{2}\d{2}\d{2}/.test(entry.name))
  .map((entry) => entry.name)
  .sort()

const migrations = await Promise.all(
  migrationDirs.map(async (name) => {
    const file = path.join(dir, "migration", name, "migration.sql")
    const sql = await Bun.file(file).text()
    const match = /^(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})/.exec(name)
    const timestamp = match
      ? Date.UTC(
        Number(match[1]),
        Number(match[2]) - 1,
        Number(match[3]),
        Number(match[4]),
        Number(match[5]),
        Number(match[6]),
      )
      : 0
    return { sql, timestamp, name }
  }),
)
console.log(`Loaded ${migrations.length} migrations`)

const singleFlag = process.argv.includes("--single")
const baselineFlag = process.argv.includes("--baseline")
const skipInstall = process.argv.includes("--skip-install")
const sourcemapsFlag = process.argv.includes("--sourcemaps")
const plugin = createSolidTransformPlugin()

// cypher_change start - codebase indexing
async function copyTreeSitterWasms(outputDir: string) {
  const runtimeWasmPath = require.resolve("web-tree-sitter/tree-sitter.wasm")
  const languagePackagePath = require.resolve("tree-sitter-wasms/package.json")
  const languageWasmDir = path.join(path.dirname(languagePackagePath), "out")
  const targetDir = path.join(outputDir, "tree-sitter")

  await fs.promises.mkdir(targetDir, { recursive: true })
  await fs.promises.copyFile(runtimeWasmPath, path.join(targetDir, "tree-sitter.wasm"))

  const languageWasmFiles = (await fs.promises.readdir(languageWasmDir)).filter((file) => file.endsWith(".wasm"))

  await Promise.all(
    languageWasmFiles.map((file) => fs.promises.copyFile(path.join(languageWasmDir, file), path.join(targetDir, file))),
  )

  console.log(`copied ${languageWasmFiles.length + 1} tree-sitter wasm files to ${targetDir}`)
}

async function buildCypherConsole() {
  const app = path.resolve(dir, "../cypher-console")
  const out = path.join(app, "dist")
  console.log("building Cypher Console")
  const proc = Bun.spawn([process.execPath, "run", "build"], {
    cwd: app,
    env: { ...process.env, CYPHER_CONSOLE_BASE: "/console/" },
    stdout: "inherit",
    stderr: "inherit",
    windowsHide: true,
  })
  const code = await proc.exited
  if (code !== 0) throw new Error(`Cypher Console build failed with exit code ${code}`)
  return out
}

async function copyCypherConsole(input: string, outputDir: string) {
  const target = path.join(outputDir, "console")
  await fs.promises.rm(target, { recursive: true, force: true })
  await fs.promises.cp(input, target, { recursive: true })
  console.log(`copied Cypher Console assets to ${target}`)
}

function smokeEnv(root: string) {
  const env = { ...process.env }
  delete env.CYPHER_MODELS_PATH
  delete env.CYPHER_MODELS_URL
  delete env.CYPHER_CONFIG
  delete env.CYPHER_CONFIG_DIR
  return {
    ...env,
    XDG_DATA_HOME: path.join(root, "data"),
    XDG_CACHE_HOME: path.join(root, "cache"),
    XDG_CONFIG_HOME: path.join(root, "config"),
    XDG_STATE_HOME: path.join(root, "state"),
    CYPHER_DISABLE_MODELS_FETCH: "1",
    CYPHER_DISABLE_PROJECT_CONFIG: "1",
    CYPHER_CONFIG_CONTENT: JSON.stringify({ enabled_providers: ["anthropic"] }),
    ANTHROPIC_API_KEY: "dummy",
  }
}

async function smokeModels(binaryPath: string) {
  const root = await fs.promises.mkdtemp(path.join(os.tmpdir(), "cypher-models-"))
  try {
    const out = await $`${binaryPath} --pure models anthropic`.env(smokeEnv(root)).text()
    if (out.split(/\r?\n/).some((line) => line.startsWith("anthropic/"))) return
    throw new Error("Compiled binary did not list Anthropic models from the embedded snapshot")
  } finally {
    await fs.promises
      .rm(root, { recursive: true, force: true })
      .catch((err) => console.warn(`Failed to remove smoke test directory ${root}`, err))
  }
}

// Cypher dropped the packages/app web UI. Kept here as a commented reference so future upstream merges
// can see the deliberate divergence rather than treating a re-add as a clean re-introduction.
// const createEmbeddedWebUIBundle = async () => {
//   console.log(`Building Web UI to embed in the binary`)
//   const appDir = path.join(import.meta.dirname, "../../app")
//   const dist = path.join(appDir, "dist")
//   await $`bun run --cwd ${appDir} build`
//   const files = (await Array.fromAsync(new Bun.Glob("**/*").scan({ cwd: dist })))
//     .map((file) => file.replaceAll("\\", "/"))
//     .filter((file) => !file.endsWith(".map"))
//     .sort()
//   const imports = files.map((file, i) => {
//     const spec = path.relative(dir, path.join(dist, file)).replaceAll("\\", "/")
//     return `import file_${i} from ${JSON.stringify(spec.startsWith(".") ? spec : `./${spec}`)} with { type: "file" };`
//   })
//   const entries = files.map((file, i) => `  ${JSON.stringify(file)}: file_${i},`)
//   return [
//     `// Import all files as file_$i with type: "file"`,
//     ...imports,
//     `// Export with original mappings`,
//     `export default {`,
//     ...entries,
//     `}`,
//   ].join("\n")
// }
// cypher_change end

const allTargets: {
  os: string
  arch: "arm64" | "x64"
  abi?: "musl"
  avx2?: false
}[] = [
    {
      os: "linux",
      arch: "arm64",
    },
    {
      os: "linux",
      arch: "x64",
    },
    {
      os: "linux",
      arch: "x64",
      avx2: false,
    },
    {
      os: "linux",
      arch: "arm64",
      abi: "musl",
    },
    {
      os: "linux",
      arch: "x64",
      abi: "musl",
    },
    {
      os: "linux",
      arch: "x64",
      abi: "musl",
      avx2: false,
    },
    {
      os: "darwin",
      arch: "arm64",
    },
    {
      os: "darwin",
      arch: "x64",
    },
    {
      os: "darwin",
      arch: "x64",
      avx2: false,
    },
    {
      os: "win32",
      arch: "arm64",
    },
    {
      os: "win32",
      arch: "x64",
    },
    {
      os: "win32",
      arch: "x64",
      avx2: false,
    },
  ]

const targets = singleFlag
  ? allTargets.filter((item) => {
    if (item.os !== process.platform || item.arch !== process.arch) {
      return false
    }

    // When building for the current platform, prefer a single native binary by default.
    // Baseline binaries require additional Bun artifacts and can be flaky to download.
    if (item.avx2 === false) {
      return baselineFlag
    }

    // also skip abi-specific builds for the same reason
    if (item.abi !== undefined) {
      return false
    }

    return true
  })
  : allTargets

await $`rm -rf dist`
// cypher_change start
const cypherConsoleDist = await buildCypherConsole()
const cypherSandboxWorker = await CypherSandboxWorker.bundle()
// cypher_change end

const binaries: Record<string, string> = {}
if (!skipInstall) {
  await $`bun install --os="*" --cpu="*" @opentui/core@${pkg.dependencies["@opentui/core"]}`
  await $`bun install --os="*" --cpu="*" @parcel/watcher@${pkg.dependencies["@parcel/watcher"]}`
}
for (const item of targets) {
  const name = [
    pkg.name,
    // changing to win32 flags npm for some reason
    item.os === "win32" ? "windows" : item.os,
    item.arch,
    item.avx2 === false ? "baseline" : undefined,
    item.abi === undefined ? undefined : item.abi,
  ]
    .filter(Boolean)
    .join("-")

  console.log(`building ${name}`)
  await $`mkdir -p dist/${name}/bin`
  // cypher_change start
  const bwrap =
    item.os === "linux" && process.env.CYPHER_SKIP_BUNDLED_BWRAP !== "1"
      ? await stageBubblewrap(item.arch, path.resolve(dir, `dist/${name}/bin`))
      : undefined
  // cypher_change end

  const localPath = path.resolve(dir, "node_modules/@opentui/core/parser.worker.js")
  const rootPath = path.resolve(dir, "../../node_modules/@opentui/core/parser.worker.js")
  const parserWorker = fs.realpathSync(fs.existsSync(localPath) ? localPath : rootPath)
  const workerPath = "./src/cli/cmd/tui/worker.ts"
  // cypher_change start
  const sessionExportWorkerPath = "./src/cypher/session-export/worker.ts"
  const indexingWorkerPath = "./src/cypher/indexing-worker.ts"
  // cypher_change end

  // Use platform-specific bunfs root path based on target OS
  const bunfsRoot = item.os === "win32" ? "B:/~BUN/root/" : "/$bunfs/root/"
  const workerRelativePath = path.relative(dir, parserWorker).replaceAll("\\", "/")

  await Bun.build({
    conditions: ["bun", "node"], // cypher_change - port anomalyco/opencode#30873; current form from #31566
    tsconfig: "./tsconfig.json",
    plugins: [plugin],
    // cypher_change start - skip sourcemaps for release builds (each .js.map adds ~50 MB per target → ~600 MB total)
    sourcemap: Script.release ? "none" : "external",
    external: ["node-gyp", ...LanceDBRuntime.external],
    // cypher_change end
    format: "esm",
    minify: true,
    // cypher_change start - disable code-splitting to avoid a Bun 1.3.14 codegen bug.
    // With splitting:true Bun emits cross-chunk re-exports like `import{vn as G9}` whose
    // binding isn't top-level, so the compiled binary crashes at startup on the baseline
    // target: "SyntaxError: Exported binding 'G9' needs to refer to a top-level declared
    // variable." (Bun oven-sh/bun#25621, #5344, #7265; also opencode#23349). Fixed upstream
    // in Bun#26089, post-1.3.14. Splitting only deduped shared code between the entrypoints;
    // turning it off inlines per entrypoint and produces a valid binary.
    splitting: false,
    // cypher_change end
    compile: {
      autoloadBunfig: false,
      autoloadDotenv: false,
      autoloadTsconfig: true,
      autoloadPackageJson: true,
      target: name.replace(pkg.name, "bun") as any,
      // cypher_change start
      outfile: `dist/${name}/bin/cypher`,
      execArgv: [`--user-agent=cypher/${Script.version}`, "--use-system-ca", "--"],
      // cypher_change end
      windows: {},
    },
    // cypher_change start - packages/app was removed; no embedded web UI
    files: {},
    entrypoints: ["./src/index.ts", parserWorker, workerPath, sessionExportWorkerPath, indexingWorkerPath],
    // cypher_change end
    define: {
      CYPHER_VERSION: `'${Script.version}'`,
      CYPHER_MIGRATIONS: JSON.stringify(migrations),
      CYPHER_MODELS_DEV: generated.modelsData,
      OTUI_TREE_SITTER_WORKER_PATH: bunfsRoot + workerRelativePath,
      CYPHER_WORKER_PATH: workerPath,
      // cypher_change start
      CYPHER_SESSION_EXPORT_WORKER_PATH: sessionExportWorkerPath,
      CYPHER_INDEXING_WORKER_PATH: indexingWorkerPath,
      CYPHER_SANDBOX_MUTATION_WORKER_PATH: JSON.stringify(CypherSandboxWorker.filename),
      // cypher_change end
      CYPHER_CHANNEL: `'${Script.channel}'`,
      CYPHER_LIBC: item.os === "linux" ? `'${item.abi ?? "glibc"}'` : "",
      // cypher_change start
      CYPHER_BWRAP_SHA256: bwrap ? `'${bwrap}'` : "undefined",
      CYPHER_BUILD_KIND: Script.release ? `'release'` : `'source'`,
      // cypher_change end
    },
  })

  // cypher_change start
  await copyTreeSitterWasms(path.resolve(dir, `dist/${name}/bin`))
  await copyCypherConsole(cypherConsoleDist, path.resolve(dir, `dist/${name}/bin`))
  await CypherSandboxWorker.copy(cypherSandboxWorker, path.resolve(dir, `dist/${name}/bin`))

  if (item.os === "linux") {
    const interpreters: Record<string, string> = {
      x64: "/lib64/ld-linux-x86-64.so.2",
      arm64: "/lib/ld-linux-aarch64.so.1",
      "x64-musl": "/lib/ld-musl-x86_64.so.1",
      "arm64-musl": "/lib/ld-musl-aarch64.so.1",
    }
    const key = item.abi === "musl" ? `${item.arch}-musl` : item.arch
    const interpreter = interpreters[key]
    if (interpreter) {
      try {
        await $`patchelf --set-interpreter ${interpreter} dist/${name}/bin/cypher`
        console.log(`patched interpreter for ${name} -> ${interpreter}`)
      } catch {
        console.warn(`patchelf not available, skipping interpreter fix for ${name}`)
      }
    }
  }
  // cypher_change end

  // Smoke test: only run if binary is for current platform
  if (item.os === process.platform && item.arch === process.arch && !item.abi) {
    const binaryPath = `dist/${name}/bin/cypher` // cypher_change
    console.log(`Running smoke test: ${binaryPath} --version`)
    try {
      const versionOutput = await $`${binaryPath} --version`.text()
      console.log(`Smoke test passed: ${versionOutput.trim()}`)
      // cypher_change start
      console.log(`Running smoke test: ${binaryPath} --pure models anthropic`)
      await smokeModels(binaryPath)
      console.log("Models snapshot smoke test passed")
      await CypherSandboxWorker.smoke(binaryPath)
      console.log("Cypher sandbox mutation worker smoke test passed")
      // cypher_change end
      // cypher_change start
    } catch (e) {
      console.error(`Smoke test failed for ${name}:`, e)
      process.exit(1)
    }
  }
  // cypher_change end

  await $`rm -rf ./dist/${name}/bin/tui`
  // cypher_change start
  if (bwrap) {
    const licenses = path.resolve(dir, `dist/${name}/bin/licenses/bubblewrap`)
    const content = await Promise.all([
      Bun.file(path.resolve(dir, "../../LICENSE")).text(),
      Bun.file(path.join(licenses, "NOTICE")).text(),
      Bun.file(path.join(licenses, "COPYING")).text(),
      Bun.file(path.join(licenses, "MUSL-COPYRIGHT")).text(),
    ])
    await Bun.write(`dist/${name}/LICENSE`, content.join("\n\n---\n\n"))
  }
  // cypher_change end
  await Bun.file(`dist/${name}/package.json`).write(
    JSON.stringify(
      {
        name,
        version: Script.version,
        license: bwrap ? "SEE LICENSE IN LICENSE" : pkg.license, // cypher_change
        preferUnplugged: true,
        os: [item.os],
        cpu: [item.arch],
        // cypher_change start
        keywords: pkg.keywords,
        private: pkg.private,
        repository: {
          type: "git",
          url: "https://github.com/Cypher-Org/cypher",
        },
        // cypher_change end
      },
      null,
      2,
    ),
  )
  binaries[name] = Script.version
}

if (Script.release) {
  const archives: string[] = [] // cypher_change
  for (const key of Object.keys(binaries)) {
    const archive = key.replace(pkg.name, "cypher") // cypher_change
    if (key.includes("linux")) {
      // cypher_change start
      const out = path.resolve("dist", `${archive}.tar.gz`)
      await $`tar -czf ${out} *`.cwd(`dist/${key}/bin`)
      archives.push(out)
      // cypher_change end
    } else {
      // cypher_change start
      const out = path.resolve("dist", `${archive}.zip`)
      await $`zip -r ${out} *`.cwd(`dist/${key}/bin`)
      archives.push(out)
      // cypher_change end
    }
  }
  await $`gh release upload v${Script.version} ${archives} --clobber` // cypher_change
}

export { binaries }
