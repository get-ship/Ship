#!/usr/bin/env bun
// grade.ts — score each lane's views/create-space/ output.
//
// Run by the `sui-eval` skill after `dispatch-eval.sh` lanes finish. We
// discover lane worktrees by `-haiku`/`-sonnet` suffix (no daemon, no
// orchestrator), then for each lane:
//
//   1. compile every .sui file under the view dir via sui-compiler
//   2. lint via sui-lint
//   3. structural pass: required signals + control-flow + token-only CSS
//   4. emit one JSON row per lane → .artifacts/sui-eval-runs/<run-id>.jsonl
//
// Discovery is best-effort and fail-loud — if a lane's worktree isn't
// reachable the row records the discovery gap rather than silently passing.

import { execSync, spawnSync } from 'node:child_process'
import { readdirSync, statSync, existsSync, readFileSync, mkdirSync, writeFileSync } from 'node:fs'
import { join, resolve } from 'node:path'

const RUN_ID = process.env.SHIP_PIPELINE_RUN_ID ?? `local-${process.env.SHIP_ACTOR_ID ?? 'unknown'}`

// The view the lanes were asked to build + the structural rubric to score it
// against. Both are env-overridable so the cheap smoke pipeline (haiku →
// views/smoke-counter) can drive the SAME grade → record → view chain as the
// live create-space eval without rewriting the scorer. Defaults preserve the
// create-space rubric.
const VIEW_PATH = process.env.SUI_EVAL_VIEW_PATH ?? 'views/create-space'
const REQUIRED_TEMPLATE_FRAGMENTS = process.env.SUI_EVAL_REQUIRED_FRAGMENTS
  ? process.env.SUI_EVAL_REQUIRED_FRAGMENTS.split(',').map((s) => s.trim()).filter(Boolean)
  : [
      // Step 0 / welcome
      '{#if', '{#match', '{:case',
      // Form / input
      'bind:value',
      // Composition
      'on:click',
    ]
const REQUIRED_MANIFEST_FIELDS = ['name', 'path', 'title', 'auth']
const FORBIDDEN_IN_CSS = /#[0-9a-fA-F]{3,8}|rgb\(|\b\d+px\b/g

interface LaneScore {
  laneId: string
  worktree: string
  compile: { passed: boolean; stderr: string }
  lint: { errors: number; warnings: number; output: string }
  structural: {
    manifestOk: boolean
    requiredFragments: { found: string[]; missing: string[] }
    cssTokenViolations: string[]
    viewFiles: string[]
  }
  composite: number
}

function findLaneWorktrees(): { laneId: string; path: string }[] {
  // The orchestrator names lane worktrees after `{parent_slug}-{lane_id}` under
  // SHIP_WORKTREE_DIR (~/.ship/worktrees by default). We scan siblings of our
  // own worktree.
  const cwd = process.cwd()
  const base = process.env.SHIP_WORKTREE_DIR
    ?? join(process.env.HOME ?? '', '.ship', 'worktrees')
  const lanes: { laneId: string; path: string }[] = []
  if (!existsSync(base)) {
    console.warn(`grade: no worktrees dir at ${base}`)
    return lanes
  }
  for (const entry of readdirSync(base)) {
    const p = join(base, entry)
    if (!statSync(p).isDirectory()) continue
    // Match `sui-eval-<runId>-<laneId>` or `*-haiku` / `*-sonnet` shape.
    const m = entry.match(/-(haiku|sonnet)$/)
    if (!m) continue
    if (existsSync(join(p, VIEW_PATH))) lanes.push({ laneId: m[1]!, path: p })
  }
  if (lanes.length === 0) {
    console.warn(`grade: discovered 0 lane worktrees under ${base}`)
    console.warn(`grade: falling back to current cwd as a single anonymous lane`)
    if (existsSync(join(cwd, VIEW_PATH))) lanes.push({ laneId: 'cwd', path: cwd })
  }
  return lanes
}

function scoreLane(laneId: string, worktree: string): LaneScore {
  const viewDir = join(worktree, VIEW_PATH)
  const viewFiles = collectViewFiles(viewDir)

  const compile = compileView(worktree, viewFiles)
  const lint = lintView(worktree)
  const structural = structuralCheck(worktree, viewFiles)

  // Composite: compile 0.40 + lint 0.20 + structural 0.40 (weights tunable;
  // we surface them in the jsonl so the comparison view can re-weight later).
  const compileScore = compile.passed ? 1 : 0
  const lintScore = lint.errors === 0
    ? Math.max(0, 1 - lint.warnings * 0.05)
    : 0
  const structuralScore = structuralFraction(structural)
  const composite = 0.40 * compileScore + 0.20 * lintScore + 0.40 * structuralScore

  return { laneId, worktree, compile, lint, structural, composite }
}

function collectViewFiles(dir: string): string[] {
  const out: string[] = []
  if (!existsSync(dir)) return out
  function walk(d: string): void {
    for (const entry of readdirSync(d)) {
      const p = join(d, entry)
      if (statSync(p).isDirectory()) walk(p)
      else out.push(p)
    }
  }
  walk(dir)
  return out
}

function compileView(worktree: string, files: string[]): LaneScore['compile'] {
  const suiFiles = files.filter((p) => p.endsWith('.sui'))
  if (suiFiles.length === 0) return { passed: false, stderr: 'no .sui files emitted' }
  let stderr = ''
  for (const f of suiFiles) {
    const isComponent = f.includes('/_components/')
    const compName = isComponent ? deriveComponentName(f) : undefined
    const args = ['run', '-p', 'sui-compiler', '--', 'build', f]
    if (compName) args.push('--component-name', compName)
    const r = spawnSync('cargo', args, { cwd: worktree, encoding: 'utf8' })
    if (r.status !== 0) {
      stderr += `[${f}] ${r.stderr ?? ''}\n`
    }
  }
  return { passed: stderr === '', stderr }
}

function deriveComponentName(suiPath: string): string | undefined {
  const m = suiPath.match(/_components\/([\w-]+)\/[\w-]+\.sui$/)
  return m?.[1]
}

function lintView(worktree: string): LaneScore['lint'] {
  // sui-lint is a [lib] crate with no [[bin]] target — it runs as a library
  // inside `sui-compiler build`'s check phase. The compile step IS the lint
  // step; parse errors + warnings from the compiler's output.
  const r = spawnSync(
    'cargo',
    ['run', '-p', 'sui-compiler', '--', 'build', `${VIEW_PATH}/page.sui`],
    { cwd: worktree, encoding: 'utf8' },
  )
  const text = (r.stdout ?? '') + (r.stderr ?? '')
  const errors = (text.match(/\berror:/g) ?? []).length
  const warnings = (text.match(/\bwarning:/g) ?? []).length
  return { errors, warnings, output: text }
}

function structuralCheck(worktree: string, files: string[]): LaneScore['structural'] {
  const manifestPath = join(worktree, VIEW_PATH, 'manifest.jsonc')
  let manifestOk = false
  if (existsSync(manifestPath)) {
    try {
      const raw = readFileSync(manifestPath, 'utf8').replace(/\/\/.*$/gm, '')
      const m = JSON.parse(raw) as Record<string, unknown>
      manifestOk = REQUIRED_MANIFEST_FIELDS.every((k) => k in m)
    } catch { manifestOk = false }
  }

  const templateText = files
    .filter((p) => p.endsWith('.sui'))
    .map((p) => readFileSync(p, 'utf8'))
    .join('\n')
  const found = REQUIRED_TEMPLATE_FRAGMENTS.filter((s) => templateText.includes(s))
  const missing = REQUIRED_TEMPLATE_FRAGMENTS.filter((s) => !templateText.includes(s))

  const cssText = files
    .filter((p) => p.endsWith('.css'))
    .map((p) => readFileSync(p, 'utf8'))
    .join('\n')
    .replace(/\/\*[\s\S]*?\*\//g, '')
  const cssTokenViolations = Array.from(new Set(cssText.match(FORBIDDEN_IN_CSS) ?? []))

  return {
    manifestOk,
    requiredFragments: { found, missing },
    cssTokenViolations,
    viewFiles: files.map((p) => p.replace(worktree + '/', '')),
  }
}

function structuralFraction(s: LaneScore['structural']): number {
  if (s.viewFiles.length === 0) return 0
  const reqTotal = REQUIRED_TEMPLATE_FRAGMENTS.length
  const reqHits = s.requiredFragments.found.length
  const cssClean = s.cssTokenViolations.length === 0 ? 1 : 0
  const manifest = s.manifestOk ? 1 : 0
  // Equal weight: template fragments, css purity, manifest presence.
  return (reqHits / reqTotal) * 0.5 + cssClean * 0.25 + manifest * 0.25
}

// --- main -----------------------------------------------------------------

const lanes = findLaneWorktrees()
console.log(`grade: scoring ${lanes.length} lane(s) for run ${RUN_ID}`)
const results = lanes.map(({ laneId, path }) => scoreLane(laneId, path))

const outDir = resolve('.artifacts/sui-eval-runs')
mkdirSync(outDir, { recursive: true })
const outFile = join(outDir, `${RUN_ID}.jsonl`)
const lines = results.map((r) => JSON.stringify({ runId: RUN_ID, ...r })).join('\n') + '\n'
writeFileSync(outFile, lines)

console.log(`grade: wrote ${results.length} row(s) → ${outFile}`)
for (const r of results) {
  const tag = r.compile.passed ? 'ok' : 'fail'
  console.log(`  ${r.laneId.padEnd(8)} composite=${r.composite.toFixed(2)} compile=${tag} lint=${r.lint.errors}e/${r.lint.warnings}w`)
}

// Exit 0 even if a lane scored zero — `grade` is a measurement, not a gate.
// The `record` step that follows is what makes the data durable.
process.exit(0)
