#!/usr/bin/env bun
/**
 * Skill-docs drift check.
 *
 * Exits non-zero when the on-disk Astro reference tree
 * (`apps/docs/src/content/docs/reference/`) disagrees with what
 * `collect-skill-docs.ts` would produce from the source tree
 * (`docs/<skill>/references/docs/`).
 *
 * Failure mode: a contributor edited the generated tree directly, or edited
 * a source page and forgot to re-run the collector before launching the dev
 * server. The diff tells them which files differ.
 *
 * Strategy: rerun the collector with COLLECT_DOCS_OUT pointed at a tempdir,
 * then compare file sets and per-file contents against the actual tree.
 */
import { spawnSync } from "node:child_process"
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { join, relative, resolve } from "node:path"

const SCRIPT_DIR = resolve(new URL(".", import.meta.url).pathname)
const REPO_ROOT = resolve(SCRIPT_DIR, "..", "..", "..")
const COLLECTOR = resolve(SCRIPT_DIR, "collect-skill-docs.ts")
const ACTUAL_DIR = resolve(
  REPO_ROOT,
  "apps/docs/src/content/docs/reference",
)

function fatal(msg: string): never {
  console.error(`check-skill-docs-drift: ${msg}`)
  process.exit(2)
}

function walk(root: string): Map<string, string> {
  const out = new Map<string, string>()
  if (!existsSync(root)) return out
  const stack: string[] = [root]
  while (stack.length > 0) {
    const dir = stack.pop()!
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const p = join(dir, entry.name)
      if (entry.isDirectory()) {
        stack.push(p)
      } else if (entry.isFile()) {
        out.set(relative(root, p), readFileSync(p, "utf8"))
      }
    }
  }
  return out
}

function runCollectorTo(outputDir: string): void {
  const res = spawnSync(
    "bun",
    [COLLECTOR],
    {
      cwd: REPO_ROOT,
      encoding: "utf8",
      env: {
        ...process.env,
        AUDIENCE: "internal",
        COLLECT_DOCS_OUT: outputDir,
      },
      maxBuffer: 32 * 1024 * 1024,
    },
  )
  if (res.status !== 0) {
    fatal(
      `collector failed (exit ${res.status}):\n${res.stderr.slice(0, 4000)}`,
    )
  }
}

function main(): void {
  if (!existsSync(ACTUAL_DIR)) {
    fatal(
      `actual output dir missing: ${ACTUAL_DIR}\n` +
        "Run `pnpm -C apps/docs run collect` (or `just docs-regen`) first.",
    )
  }
  const tmp = mkdtempSync(join(tmpdir(), "ship-docs-drift-"))
  try {
    runCollectorTo(tmp)
    const expected = walk(tmp)
    const actual = walk(ACTUAL_DIR)

    const expectedKeys = new Set(expected.keys())
    const actualKeys = new Set(actual.keys())

    const missing = [...expectedKeys].filter((k) => !actualKeys.has(k)).sort()
    const extra = [...actualKeys].filter((k) => !expectedKeys.has(k)).sort()
    const changed: string[] = []
    for (const k of expectedKeys) {
      if (actualKeys.has(k) && expected.get(k) !== actual.get(k)) {
        changed.push(k)
      }
    }
    changed.sort()

    if (missing.length === 0 && extra.length === 0 && changed.length === 0) {
      console.log(
        `Skill-docs tree matches source (${expectedKeys.size} files).`,
      )
      process.exit(0)
    }

    console.error("Skill-docs drift detected.\n")
    console.error(
      `Source: docs/<skill>/references/docs/\n` +
        `Generated: ${ACTUAL_DIR}\n\n` +
        "Regenerate with:\n" +
        "  pnpm -C apps/docs run collect\n" +
        "(or `just docs-regen`).\n",
    )
    if (missing.length > 0) {
      console.error(`\nMissing in actual (${missing.length}):`)
      for (const k of missing) console.error(`  - ${k}`)
    }
    if (extra.length > 0) {
      console.error(`\nExtra in actual (${extra.length}):`)
      for (const k of extra) console.error(`  + ${k}`)
    }
    if (changed.length > 0) {
      console.error(`\nContent differs (${changed.length}):`)
      for (const k of changed) console.error(`  ~ ${k}`)
    }
    process.exit(1)
  } finally {
    try { rmSync(tmp, { recursive: true, force: true }) } catch {}
  }
}

main()
