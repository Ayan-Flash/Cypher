import { Flag } from "@opencode-ai/core/flag/flag"
import { Effect } from "effect"
import path from "path"

const preserveExerciseGlobalRoot = !!process.env.CYPHER_HTTPAPI_EXERCISE_GLOBAL
export const exerciseGlobalRoot =
  process.env.CYPHER_HTTPAPI_EXERCISE_GLOBAL ??
  path.join(process.env.TMPDIR ?? "/tmp", `opencode-httpapi-global-${process.pid}`)
process.env.XDG_DATA_HOME = path.join(exerciseGlobalRoot, "data")
process.env.XDG_CONFIG_HOME = path.join(exerciseGlobalRoot, "config")
process.env.XDG_STATE_HOME = path.join(exerciseGlobalRoot, "state")
process.env.XDG_CACHE_HOME = path.join(exerciseGlobalRoot, "cache")
process.env.CYPHER_DISABLE_SHARE = "true"
process.env.CYPHER_DISABLE_SESSION_INGEST = "true" // cypher_change - isolate the exerciser from async Cypher session sync
export const exerciseConfigDirectory = path.join(exerciseGlobalRoot, "config", "opencode")
export const exerciseDataDirectory = path.join(exerciseGlobalRoot, "data", "cypher") // cypher_change

const preserveExerciseDatabase = !!process.env.CYPHER_HTTPAPI_EXERCISE_DB
export const exerciseDatabasePath =
  process.env.CYPHER_HTTPAPI_EXERCISE_DB ??
  path.join(process.env.TMPDIR ?? "/tmp", `opencode-httpapi-exercise-${process.pid}.db`)
process.env.CYPHER_DB = exerciseDatabasePath
Flag.CYPHER_DB = exerciseDatabasePath

export const original = {
  CYPHER_SERVER_PASSWORD: Flag.CYPHER_SERVER_PASSWORD,
  CYPHER_SERVER_USERNAME: Flag.CYPHER_SERVER_USERNAME,
}

export const cleanupExercisePaths = Effect.promise(async () => {
  const fs = await import("fs/promises")
  if (!preserveExerciseDatabase) {
    await Promise.all(
      [exerciseDatabasePath, `${exerciseDatabasePath}-wal`, `${exerciseDatabasePath}-shm`].map((file) =>
        fs.rm(file, { force: true }).catch(() => undefined),
      ),
    )
  }
  if (!preserveExerciseGlobalRoot)
    await fs.rm(exerciseGlobalRoot, { recursive: true, force: true }).catch(() => undefined)
})
