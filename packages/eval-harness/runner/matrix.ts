#!/usr/bin/env bun
// `bun run eval:matrix -- --cases all --skills cardinals,stripped` —
// matrix runner. Composes cells of (case × skill × profile), dispatches
// each through the cli runner, aggregates into a summary table.
//
// v0 scope: shells out to `cli.ts` per cell, captures the resulting
// cell.json, rolls up into `results/<run-id>/summary.md`. A future cut
// runs the cells in parallel (per-process isolation matters when the
// backend dispatches real worktrees).

import { execFileSync } from 'node:child_process'
import { readdirSync, readFileSync, mkdirSync, writeFileSync, statSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

import { listCases } from './load-case'
import type { ScoreCard } from './types'

const __filename = fileURLToPath(import.meta.url)
const harnessRoot = resolve(dirname(__filename), '..')

interface MatrixOpts {
  cases: string[]
  skills: string[]
  profiles: string[]
  backend: string
}

function parseArgs(argv: string[]): MatrixOpts {
  const opts: MatrixOpts = {
    cases: ['all'],
    skills: ['main'],
    profiles: ['default'],
    backend: 'stub',
  }
  for (let i = 0; i < argv.length; i++) {
    const k = argv[i]
    const v = argv[i + 1]
    if (k === '--cases' && v) { opts.cases = v.split(','); i++ }
    else if (k === '--skills' && v) { opts.skills = v.split(','); i++ }
    else if (k === '--profiles' && v) { opts.profiles = v.split(','); i++ }
    else if (k === '--backend' && v) { opts.backend = v; i++ }
  }
  if (opts.cases.length === 1 && opts.cases[0] === 'all') {
    opts.cases = listCases()
  }
  return opts
}

interface Cell {
  caseId: string
  skill: string
  profile: string
  score: ScoreCard | null
  error?: string
}

function runCell(caseId: string, skill: string, profile: string, backend: string): Cell {
  try {
    execFileSync(
      'bun',
      ['runner/cli.ts', '--case', caseId, '--backend', backend, '--skill', skill, '--profile', profile],
      { cwd: harnessRoot, encoding: 'utf8', stdio: 'inherit' },
    )
    // Cell.json under results/<latest>; find it by the case+backend tag
    // in the run-id timestamp.
    const latest = latestRunMatching(caseId, backend)
    const cell = JSON.parse(readFileSync(resolve(latest, 'cell.json'), 'utf8')) as { score: ScoreCard }
    return { caseId, skill, profile, score: cell.score }
  } catch (e) {
    return { caseId, skill, profile, score: null, error: e instanceof Error ? e.message : String(e) }
  }
}

function latestRunMatching(caseId: string, backend: string): string {
  const root = resolve(harnessRoot, 'results')
  const candidates = readdirSync(root)
    .filter((d) => d.includes(`-${caseId}-${backend}`))
    .map((d) => resolve(root, d))
    .filter((d) => statSync(d).isDirectory())
  candidates.sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs)
  if (candidates.length === 0) throw new Error(`no run dir for ${caseId}/${backend}`)
  return candidates[0]!
}

function summarize(cells: Cell[]): string {
  const header = `| case | skill | profile | compile | lint errors | structural | composite |\n|---|---|---|---|---|---|---|`
  const rows = cells.map((c) => {
    if (!c.score) {
      return `| ${c.caseId} | ${c.skill} | ${c.profile} | — | — | — | error: ${c.error?.slice(0, 40) ?? ''} |`
    }
    const s = c.score
    return `| ${c.caseId} | ${c.skill} | ${c.profile} | ${s.compile.passed ? '✓' : '✗'} | ${s.lint.errors} | ${s.structural.matched}/${s.structural.matched + s.structural.missing.length} | ${(s.composite * 100).toFixed(1)}% |`
  })
  return [header, ...rows].join('\n')
}

async function main(): Promise<void> {
  const opts = parseArgs(process.argv.slice(2))
  const cells: Cell[] = []
  for (const caseId of opts.cases) {
    for (const skill of opts.skills) {
      for (const profile of opts.profiles) {
        console.log(`\n[matrix] running case=${caseId} skill=${skill} profile=${profile}`)
        cells.push(runCell(caseId, skill, profile, opts.backend))
      }
    }
  }

  const runId = `matrix-${new Date().toISOString().replace(/[:.]/g, '-')}`
  const runDir = resolve(harnessRoot, 'results', runId)
  mkdirSync(runDir, { recursive: true })
  writeFileSync(resolve(runDir, 'manifest.json'), JSON.stringify(opts, null, 2))
  writeFileSync(resolve(runDir, 'summary.md'), summarize(cells) + '\n')
  console.log(`\n[matrix] summary → ${resolve(runDir, 'summary.md')}\n`)
  console.log(summarize(cells))
}

main().catch((e: unknown) => {
  console.error('[matrix] fatal:', e instanceof Error ? e.message : e)
  process.exit(1)
})
