// Load a case fixture from `corpus/<id>/` — meta.json + prompt.md +
// the resolved path to its reference under `references/<id>/`.

import { existsSync, readFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

import type { CaseFixture, CaseMeta } from './types'

const __filename = fileURLToPath(import.meta.url)
const harnessRoot = resolve(dirname(__filename), '..')

export function loadCase(id: string): CaseFixture {
  const caseDir = resolve(harnessRoot, 'corpus', id)
  if (!existsSync(caseDir)) {
    throw new Error(`eval-harness: no case at ${caseDir}`)
  }
  const metaPath = resolve(caseDir, 'meta.json')
  const promptPath = resolve(caseDir, 'prompt.md')
  if (!existsSync(metaPath) || !existsSync(promptPath)) {
    throw new Error(`eval-harness: case ${id} missing meta.json or prompt.md`)
  }

  const meta = JSON.parse(readFileSync(metaPath, 'utf8')) as CaseMeta
  validateMeta(meta, id)

  const referencePath = resolve(harnessRoot, 'references', meta.reference)
  if (!existsSync(referencePath)) {
    throw new Error(`eval-harness: case ${id} → reference '${meta.reference}' not found at ${referencePath}`)
  }

  return { meta, promptPath, referencePath }
}

export function listCases(): string[] {
  const corpusDir = resolve(harnessRoot, 'corpus')
  if (!existsSync(corpusDir)) return []
  // Lazy import to avoid `node:fs` `readdirSync` re-imports in callers.
  const { readdirSync, statSync } = require('node:fs') as typeof import('node:fs')
  return readdirSync(corpusDir).filter((name: string) =>
    statSync(resolve(corpusDir, name)).isDirectory(),
  )
}

function validateMeta(meta: CaseMeta, id: string): void {
  if (meta.id !== id) {
    throw new Error(`eval-harness: case ${id} meta.id mismatch (${meta.id} ≠ ${id})`)
  }
  if (meta.kind !== 'view' && meta.kind !== 'component') {
    throw new Error(`eval-harness: case ${id} unknown kind '${meta.kind}'`)
  }
  if (meta.kind === 'component' && !meta.tag) {
    throw new Error(`eval-harness: case ${id} kind=component but no tag`)
  }
  const w = meta.weights
  const total = w.compile + w.lint + w.structural + w.render
  if (Math.abs(total - 1.0) > 0.001) {
    throw new Error(`eval-harness: case ${id} weights must sum to 1.0 (got ${total.toFixed(3)})`)
  }
}
