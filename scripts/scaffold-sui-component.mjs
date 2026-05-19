#!/usr/bin/env node
/**
 * Scaffold a new SUI primitive.
 *
 * Usage:
 *   node scripts/scaffold-sui-component.mjs <tag> [--controller]
 *
 *   <tag> must start with `sui-` and match /^sui-[a-z][a-z0-9-]*$/.
 *   `--controller` also writes a paired `<tag>.controller.ts` with controller(def).
 *
 * Creates:
 *   - packages/components/ui-components/src/<tag>/<tag>.sui
 *   - packages/components/ui-components/src/<tag>/<tag>.controller.ts (only with --controller)
 *
 * Convention is the contract: directory IS the component, file extension
 * `.sui` is the contract, paired `.ts` is auto-detected by the compiler.
 * No `<tag>.sui.json` config file. No 1-line side-effect entry.
 *
 * After scaffolding:
 *   1. Edit <tag>.sui — replace the placeholder prop list, styles, body
 *   2. Add to packages/components/ui-components/src/index.ts:
 *        import './<tag>/dist/<tag>.compiled.js'
 *   3. Add to scripts/build-bundles.mjs componentSources:
 *        { htmlSet: 'packages/components/ui-components/src/<tag>/<tag>.sui' }
 *   4. node scripts/build-bundles.mjs --force
 */

import { existsSync, mkdirSync, writeFileSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = resolve(fileURLToPath(import.meta.url), '..', '..')

const args = process.argv.slice(2)
const tag = args.find((a) => !a.startsWith('--'))?.trim() ?? ''
const wantController = args.includes('--controller')

if (!tag) {
  console.error('usage: node scripts/scaffold-sui-component.mjs <tag> [--controller]')
  process.exit(2)
}

if (!/^sui-[a-z][a-z0-9-]*$/.test(tag)) {
  console.error(`[scaffold] tag must match /^sui-[a-z][a-z0-9-]*$/ — got "${tag}"`)
  process.exit(2)
}

const baseDir = join(repoRoot, 'packages', 'ui-components', 'src', tag)
if (existsSync(baseDir)) {
  console.error(`[scaffold] directory already exists: ${baseDir}`)
  process.exit(1)
}
mkdirSync(baseDir, { recursive: true })

const sui = `<!--
  ${tag} — SUI primitive.

  Parity vs Lit <ship-${tag.replace(/^sui-/, '')}>:
    Props:    (list)
    Slots:    (list)
    Events:   (list)
    Methods:  (list)
    ARIA:     (list)
-->

<prop name="tone" values="default|primary|info|success|warning|destructive|accent|muted" default="default" reflect />

<style>
  :host { display: inline-flex; vertical-align: middle; --tone-fg: var(--foreground); }
  :host([tone='primary'])     { --tone-fg: var(--primary); }
  :host([tone='info'])        { --tone-fg: var(--info); }
  :host([tone='success'])     { --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone-fg: var(--accent); }
  :host([tone='muted'])       { --tone-fg: var(--muted-foreground); }

  .root { color: var(--tone-fg); }
</style>

<span class="root" part="root"><slot></slot></span>
`

writeFileSync(join(baseDir, `${tag}.sui`), sui)

if (wantController) {
  const ts = `import { controller } from '@ship/sui-runtime'
import type { Scope } from './${tag}.sui'

export default controller<Scope>({
  mount: (_state, _host) => {
    return () => {}
  },
})
`
  writeFileSync(join(baseDir, `${tag}.controller.ts`), ts)
}

console.log(`[scaffold] created packages/components/ui-components/src/${tag}/`)
console.log(`[scaffold] next steps:`)
console.log(`  1. Edit ${tag}.sui — props, styles, template body, parity comment`)
console.log(`  2. Add to packages/components/ui-components/src/index.ts:`)
console.log(`       import './${tag}/dist/${tag}.compiled.js'`)
console.log(`  3. Add to scripts/build-bundles.mjs componentSources:`)
console.log(`       { htmlSet: 'packages/components/ui-components/src/${tag}/${tag}.sui' }`)
console.log(`  4. node scripts/build-bundles.mjs --force`)
console.log(`  5. cargo build -p shipd && bash scripts/dev/restart.sh`)
