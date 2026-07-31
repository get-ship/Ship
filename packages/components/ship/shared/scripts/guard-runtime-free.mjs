#!/usr/bin/env node
// Guard: @ship/shared MUST stay runtime-free.
//
// This is the gate that stops @ship/shared from becoming a dump bucket.
// A component belongs in @ship/shared only if it is purely presentational:
// it takes data + permissions as PROPS and the consumer (which owns the
// runtime) supplies them. Nothing in src/ may import the runtime, the SDK,
// or the contract — those are how a component "reaches for runtime data",
// which is exactly what this pack forbids.
//
// Self-contained on purpose: no @ship/test-setup, no vitest, no compiler.
// Runs anywhere `node`/`bun` runs. Wire it into `test`/`lint` (package.json)
// so a violating import fails CI the moment it lands.
//
// Scope: src/**/*.{ts,tsx,js,mjs,cjs,sui}. Generated *.compiled.js and *.d.ts
// are skipped — they are bundler output, not authored source, and a compiled
// presentational component legitimately links the rendering kernel at runtime
// (the host provides it). The rule is about what AUTHORS import.

import { readdirSync, readFileSync, statSync } from 'node:fs'
import { dirname, extname, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const packRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const srcRoot = join(packRoot, 'src')

// Forbidden module specifiers (exact match or `<spec>/<subpath>`).
const FORBIDDEN = [
  '@ship/sail-runtime',
  '@ship/sdk',
  '@ship/sdk-client',
  '@ship/contract',
  '@ship/contracts',
]

const SCAN_EXT = new Set(['.ts', '.tsx', '.js', '.mjs', '.cjs', '.sail'])

/** Pull every static + dynamic module specifier out of a source string. */
function specifiersOf(source) {
  const specs = []
  // import ... from '<x>' | export ... from '<x>' | import '<x>'
  const staticRe =
    /(?:import|export)\s[^'"]*?from\s*['"]([^'"]+)['"]|import\s*['"]([^'"]+)['"]/g
  // import('<x>') | require('<x>')
  const dynamicRe = /(?:import|require)\s*\(\s*['"]([^'"]+)['"]\s*\)/g
  for (const re of [staticRe, dynamicRe]) {
    let m
    while ((m = re.exec(source)) !== null) {
      const spec = m[1] ?? m[2]
      if (spec) specs.push(spec)
    }
  }
  return specs
}

function isForbidden(spec) {
  return FORBIDDEN.some((f) => spec === f || spec.startsWith(`${f}/`))
}

function walk(dir, out) {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry)
    if (statSync(full).isDirectory()) {
      walk(full, out)
      continue
    }
    if (entry.endsWith('.compiled.js') || entry.endsWith('.d.ts')) continue
    if (!SCAN_EXT.has(extname(entry))) continue
    out.push(full)
  }
}

const files = []
walk(srcRoot, files)

const violations = []
for (const file of files) {
  const source = readFileSync(file, 'utf8')
  for (const spec of specifiersOf(source)) {
    if (isForbidden(spec)) {
      violations.push({ file: relative(packRoot, file), spec })
    }
  }
}

if (violations.length > 0) {
  console.error('✗ @ship/shared runtime-free guard FAILED\n')
  for (const v of violations) {
    console.error(`  ${v.file} imports forbidden specifier "${v.spec}"`)
  }
  console.error(
    '\n@ship/shared must be runtime-free: no @ship/sail-runtime, @ship/sdk,' +
      '\n@ship/sdk-client, or contract imports. Take the data/permissions as' +
      '\nprops and let the consumer supply runtime data — or move the' +
      '\ncomponent into sui (generic primitive) or its product pack.',
  )
  process.exit(1)
}

console.log(
  `✓ @ship/shared runtime-free guard passed — scanned ${files.length} source` +
    ` file(s) under src/, no forbidden imports.`,
)
