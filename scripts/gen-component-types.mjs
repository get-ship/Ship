#!/usr/bin/env node
// Generate the adjacent `<stem>.sail.d.ts` type companions for ship-apps
// component packs (mirrors the monorepo-side gen) so controllers' `import type
// { Scope } from './x.sail'` resolves with no rootDirs into a build dir.
//
// Uses the sail-compiler BINARY (a build tool) discovered via SAIL_COMPILER /
// SUI_COMPILER or the sibling monorepo tip — it never reaches into monorepo
// *script files*. Build artifacts go to each pack's own `dist/` via `--out`;
// the `.sail.d.ts` lands adjacent.

import { execFileSync } from 'node:child_process'
import { existsSync, readdirSync, statSync } from 'node:fs'
import { dirname, resolve, basename } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const firstExisting = (ps) => ps.find((p) => p && existsSync(p)) ?? null
const ORCH = process.env.ORCH ?? process.env.SUI_ROOT ?? resolve(ROOT, '../worktrees/orch')
const COMPILER = process.env.SAIL_COMPILER ?? process.env.SUI_COMPILER ?? firstExisting([
  resolve(ORCH, 'target/release/sail-compiler'),
  resolve(ORCH, 'target/debug/sail-compiler'),
  resolve(ROOT, '../sui/target/release/sail-compiler'),
  resolve(ROOT, '../sui/target/debug/sail-compiler'),
  resolve(ROOT, '../sui/target/release/sui-compiler'),
  resolve(ROOT, '../sui/target/debug/sui-compiler'),
])
if (!COMPILER || !existsSync(COMPILER)) {
  console.error('sail-compiler not found. Set SAIL_COMPILER/SUI_COMPILER, or build it (cargo build -p sail-compiler) in orch/sui.')
  process.exit(1)
}

const arg = process.argv[2]
const PACKS = arg
  ? [resolve(arg)]
  : readdirSync(resolve(ROOT, 'packages/components/ship'))
      .map((d) => resolve(ROOT, 'packages/components/ship', d))
      .filter((d) => statSync(d).isDirectory() && existsSync(resolve(d, 'src')))

function* walkSail(dir) {
  for (const e of readdirSync(dir)) {
    const p = resolve(dir, e)
    const st = statSync(p)
    if (st.isDirectory()) { if (e === 'dist' || e === 'node_modules') continue; yield* walkSail(p) }
    else if (e.endsWith('.sail')) yield p
  }
}

let total = 0, failed = 0
for (const pack of PACKS) {
  const src = resolve(pack, 'src')
  if (!existsSync(src)) continue
  const dist = resolve(pack, 'dist')
  for (const sail of walkSail(src)) {
    const stem = basename(sail, '.sail')
    try {
      execFileSync(COMPILER, ['build', sail, '--component-name', stem, '--out', dist], { stdio: ['inherit', 'ignore', 'inherit'] })
      total++
    } catch { failed++; console.error(`  compile failed: ${sail.replace(ROOT + '/', '')}`) }
  }
}
console.log(`generated types for ${total} component(s)${failed ? `, ${failed} failed` : ''} across ${PACKS.length} pack(s)`)
