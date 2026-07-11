import { expect } from "bun:test"
import { Effect, Layer } from "effect"
import { CrossSpawnSpawner } from "@opencode-ai/core/cross-spawn-spawner"
import path from "path"
import { Skill } from "../../src/skill"
import * as CypherSkill from "../../src/cypher/skill-remove"
import { BUILTIN_SKILLS } from "../../src/cypher/skills/builtin"
import { TestInstance } from "../fixture/fixture"
import { testEffect } from "../lib/effect"

const it = testEffect(Layer.mergeAll(Skill.defaultLayer, CrossSpawnSpawner.defaultLayer))

it.instance(
  "built-in skills are present in empty project",
  () =>
    Effect.gen(function* () {
      const skill = yield* Skill.Service
      const skills = yield* skill.all()
      for (const builtin of BUILTIN_SKILLS) {
        const found = skills.find((s) => s.name === builtin.name)
        expect(found).toBeDefined()
        expect(found!.location).toBe(Skill.BUILTIN_LOCATION)
        expect(found!.description).toBe(builtin.description)
        expect(found!.content.length).toBeGreaterThan(0)
      }
    }),
  { git: true },
)

it.instance(
  "built-in skill has correct metadata",
  () =>
    Effect.gen(function* () {
      const skill = yield* Skill.Service
      const item = yield* skill.get("cypher-config")
      expect(item).toBeDefined()
      expect(item!.name).toBe("cypher-config")
      expect(item!.location).toBe(Skill.BUILTIN_LOCATION)
      expect(item!.content).toContain("cypher")
    }),
  { git: true },
)

it.instance(
  "cypher-config is protected from removal",
  () =>
    Effect.gen(function* () {
      const skill = yield* Skill.Service
      const item = yield* skill.get("cypher-config")
      expect(item).toBeDefined()
      expect(CypherSkill.builtin(item!.location)).toBe(true)
    }),
  { git: true },
)

it.instance(
  "user skill overrides built-in with same name",
  () =>
    Effect.gen(function* () {
      const instance = yield* TestInstance
      const dir = path.join(instance.directory, ".cypher", "skill", "cypher-config")
      yield* Effect.promise(() =>
        Bun.write(
          path.join(dir, "SKILL.md"),
          `---
name: cypher-config
description: User override of cypher-config.
---

# Custom cypher-config

User-provided content.
`,
        ),
      )

      const skill = yield* Skill.Service
      const item = yield* skill.get("cypher-config")
      expect(item).toBeDefined()
      expect(item!.description).toBe("User override of cypher-config.")
      expect(item!.location).not.toBe(Skill.BUILTIN_LOCATION)
      expect(item!.location).toContain(path.join("skill", "cypher-config", "SKILL.md"))
    }),
  { git: true },
)
