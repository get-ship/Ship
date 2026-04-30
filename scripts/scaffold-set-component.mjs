#!/usr/bin/env node
/**
 * Scaffold a new ship-set ported primitive.
 *
 * Usage:
 *   node scripts/scaffold-set-component.mjs <tag>
 *
 *   <tag> must start with `set-` (the migration prefix; see
 *   docs/ship-adrs/references/docs/ship-set-prefix-during-migration.md).
 *
 * Creates:
 *   - packages/ship-set-components/src/<tag>/<tag>.html.set
 *   - packages/ship-set-components/src/<tag>/<tag>.ts
 *
 * The template + .ts are minimal placeholders; replace the prop list,
 * the `<style>` block, and the body template with the real port.
 *
 * Also reminds you to:
 *   - Add the import + register* call to packages/ship-set-components/src/index.ts
 *   - Add the new port to test/all-components.test.ts
 *   - Run `node scripts/build-bundles.mjs --force` to rebuild
 */

import { existsSync, mkdirSync, writeFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = resolve(fileURLToPath(import.meta.url), '..', '..')

const tag = process.argv[2]?.trim() ?? ''
if (!tag) {
  console.error('usage: node scripts/scaffold-set-component.mjs <tag>')
  process.exit(2)
}

if (!/^set-[a-z][a-z0-9-]*$/.test(tag)) {
  console.error(
    `[scaffold] tag must match /^set-[a-z][a-z0-9-]*$/ — got "${tag}"`,
  )
  console.error(
    `[scaffold] the migration prefix is enforced; see ship-set-prefix-during-migration.md`,
  )
  process.exit(2)
}

const baseDir = join(
  repoRoot,
  'packages',
  'ship-set-components',
  'src',
  tag,
)
if (existsSync(baseDir)) {
  console.error(`[scaffold] directory already exists: ${baseDir}`)
  process.exit(1)
}
mkdirSync(baseDir, { recursive: true })

const camel = tag.replace(/-([a-z])/g, (_, c) => c.toUpperCase())
const Pascal = camel.charAt(0).toUpperCase() + camel.slice(1)
// `setStatusDot` → `SetStatusDot` for the `register*` function name.

const htmlSet = `<component name="${tag}" shadow>
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

    .root {
      /* TODO(${tag}): replace with the real component's styles. */
      color: var(--tone-fg);
    }
  </style>
</component>

<span class="root" part="root"><slot></slot></span>
`

const ts = `import { defineComponent } from '../define'
import template from './${tag}.html.set'

/** Register \`<${tag}>\`. Idempotent. */
export function register${Pascal}(): void {
  defineComponent(template)
}
`

writeFileSync(join(baseDir, `${tag}.html.set`), htmlSet)
writeFileSync(join(baseDir, `${tag}.ts`), ts)

console.log(`[scaffold] created packages/ship-set-components/src/${tag}/`)
console.log(`[scaffold] next steps:`)
console.log(`  1. Edit ${tag}.html.set — props, styles, template body`)
console.log(`  2. Edit packages/ship-set-components/src/index.ts:`)
console.log(`       import { register${Pascal} } from './${tag}/${tag}'`)
console.log(`       export { register${Pascal} }`)
console.log(`       // also call register${Pascal}() inside registerAll()`)
console.log(`  3. Add a row to packages/ship-set-components/test/all-components.test.ts`)
console.log(`  4. Run: node scripts/build-bundles.mjs --force`)
console.log(`  5. Run: pnpm --filter @ship-set/components exec vitest run`)
