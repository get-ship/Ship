#!/usr/bin/env bun
/**
 * Generate the lucide registry consumed by <sui-icon> via
 * /_sui/lucide-icons.json. Replaces the curated allowlist approach (which
 * lived in the deleted packages/ship-set/) with a dump of every lucide-static
 * icon — 1947 entries, ~300KB JSON. No allowlist drift, no missing-icon
 * blank buttons.
 *
 * Output: crates/deck/deck-server/assets/lucide-icons.json
 * Run via: `bun scripts/codegen/generate-lucide-registry.mjs`.
 */
import { readdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const repoRoot = dirname(here)
const lucideDir = join(
  repoRoot,
  'node_modules/.bun/lucide-static@1.14.0/node_modules/lucide-static/icons',
)
const outFile = join(
  repoRoot,
  'crates/deck/deck-server/assets/lucide-icons.json',
)

function extractInner(svg) {
  const stripped = svg.replace(/<!--[\s\S]*?-->/g, '').trim()
  const open = stripped.indexOf('<svg')
  const openEnd = stripped.indexOf('>', open)
  const closeStart = stripped.lastIndexOf('</svg>')
  if (open < 0 || openEnd < 0 || closeStart < 0) {
    throw new Error('malformed SVG')
  }
  return stripped
    .slice(openEnd + 1, closeStart)
    .trim()
    .replace(/\s+/g, ' ')
}

const files = readdirSync(lucideDir).filter((f) => f.endsWith('.svg'))
const out = {}
for (const f of files) {
  const name = f.replace(/\.svg$/, '')
  const raw = readFileSync(join(lucideDir, f), 'utf8')
  out[name] = extractInner(raw)
}

writeFileSync(outFile, JSON.stringify(out))
process.stdout.write(`wrote ${Object.keys(out).length} icons to ${outFile}\n`)
