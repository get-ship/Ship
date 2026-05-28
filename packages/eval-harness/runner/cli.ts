#!/usr/bin/env bun
// `bun run eval -- --case <id>` — single-case evaluator.
// Loads the case, dispatches the agent, scores the artifact, writes
// `results/<run-id>/cell.json`. v0 scope: stub backend only. The
// matrix runner (eval:matrix) wraps multiple invocations.

import { mkdirSync, rmSync, writeFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

import { loadCase } from './load-case'
import { scoreArtifact } from './score'
import { backends } from './agent'

const __filename = fileURLToPath(import.meta.url)
const harnessRoot = resolve(dirname(__filename), '..')
const workspaceRoot = resolve(harnessRoot, '..', '..')

interface CliOpts {
  case?: string
  backend?: string
  skill?: string
  profile?: string
}

function parseArgs(argv: string[]): CliOpts {
  const opts: CliOpts = {}
  for (let i = 0; i < argv.length; i++) {
    const k = argv[i]
    const v = argv[i + 1]
    if (k === '--case' && v) { opts.case = v; i++ }
    else if (k === '--backend' && v) { opts.backend = v; i++ }
    else if (k === '--skill' && v) { opts.skill = v; i++ }
    else if (k === '--profile' && v) { opts.profile = v; i++ }
  }
  return opts
}

async function main(): Promise<void> {
  const opts = parseArgs(process.argv.slice(2))
  if (!opts.case) {
    console.error('usage: bun runner/cli.ts --case <id> [--backend stub|claude] [--skill <variant>] [--profile <id>]')
    process.exit(2)
  }

  const backendName = opts.backend ?? 'stub'
  const backend = backends[backendName]
  if (!backend) {
    console.error(`unknown backend: ${backendName} (have: ${Object.keys(backends).join(', ')})`)
    process.exit(2)
  }

  const fixture = loadCase(opts.case)
  const runId = `${new Date().toISOString().replace(/[:.]/g, '-')}-${opts.case}-${backendName}`
  const resultsDir = resolve(harnessRoot, 'results', runId)
  mkdirSync(resultsDir, { recursive: true })
  const workDir = resolve(resultsDir, 'work')
  mkdirSync(workDir, { recursive: true })

  console.log(`[eval] case=${opts.case} backend=${backendName} skill=${opts.skill ?? 'main'} profile=${opts.profile ?? 'default'}`)
  console.log(`[eval] work=${workDir}`)

  const fs = await import('node:fs/promises')
  const prompt = await fs.readFile(fixture.promptPath, 'utf8')

  const response = await backend({
    caseId: opts.case,
    prompt,
    skillVariant: opts.skill ?? 'main',
    agentProfile: opts.profile ?? 'default',
    workDir,
  })

  console.log(`[eval] agent emitted ${response.artifacts.length} artifact(s)`)
  for (const a of response.artifacts) console.log(`  - ${resolve(a)}`)

  const score = scoreArtifact(fixture, response, workspaceRoot)
  writeFileSync(
    resolve(resultsDir, 'cell.json'),
    JSON.stringify({ opts, fixture: fixture.meta, response, score }, null, 2),
  )

  console.log(`\n[eval] score:`)
  console.log(`  compile:     ${score.compile.passed ? 'pass' : 'FAIL'}`)
  console.log(`  lint:        ${score.lint.errors} errors, ${score.lint.warnings} warnings`)
  console.log(`  structural:  ${score.structural.matched}/${score.structural.matched + score.structural.missing.length} required; ${score.structural.forbidden.length} forbidden hits`)
  console.log(`  composite:   ${(score.composite * 100).toFixed(1)}%`)
  console.log(`\n[eval] results → ${resultsDir}`)
  // Clean the workdir copy of artifacts — they're captured in cell.json
  // already, and leaving them around pollutes the next run's globbing.
  rmSync(workDir, { recursive: true, force: true })

  process.exit(score.compile.passed && score.lint.errors === 0 ? 0 : 1)
}

main().catch((e: unknown) => {
  console.error('[eval] fatal:', e instanceof Error ? e.message : e)
  process.exit(1)
})
