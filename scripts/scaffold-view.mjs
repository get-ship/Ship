#!/usr/bin/env node
/**
 * Scaffold a new SUI view.
 *
 * Usage:
 *   node scripts/scaffold-view.mjs <name> [--path /url] [--component <tag>]
 *
 *   <name>            view directory under views/. Lowercase ASCII + hyphens.
 *   --path            URL pattern; defaults to "/<name>".
 *   --component       optional view-private widget; tag must start with "<name>-".
 *
 * Creates:
 *   - views/<name>/page.sui
 *   - views/<name>/controller.ts
 *   - views/<name>/style.css
 *   - views/<name>/manifest.jsonc
 *   - views/<name>/_components/<tag>/<tag>.sui            (with --component)
 *   - views/<name>/_components/<tag>/<tag>.controller.ts  (with --component)
 *
 * Convention is the contract: page.sui is the route entry, paired controller
 * is plain `controller.ts` (no stem prefix), manifest.jsonc declares the route.
 * See docs/ship-adrs/references/docs/view-directory-convention.md.
 */

import { existsSync, mkdirSync, writeFileSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = resolve(fileURLToPath(import.meta.url), '..', '..')

function parseArgs(argv) {
  const positional = []
  const flags = {}
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (a === '--path' || a === '--component') {
      flags[a.slice(2)] = argv[++i]
    } else if (a.startsWith('--')) {
      flags[a.slice(2)] = true
    } else {
      positional.push(a)
    }
  }
  return { positional, flags }
}

function fail(msg, code = 1) {
  console.error(msg)
  process.exit(code)
}

const { positional, flags } = parseArgs(process.argv.slice(2))
const name = positional[0]?.trim() ?? ''
const routePath = typeof flags.path === 'string' ? flags.path : `/${name}`
const componentTag = typeof flags.component === 'string' ? flags.component.trim() : ''

if (!name) {
  fail('usage: node scripts/scaffold-view.mjs <name> [--path /url] [--component <tag>]', 2)
}
if (!/^[a-z][a-z0-9-]*$/.test(name)) {
  fail(`[scaffold] name must match /^[a-z][a-z0-9-]*$/ — got "${name}"`, 2)
}
if (!routePath.startsWith('/')) {
  fail(`[scaffold] --path must start with "/" — got "${routePath}"`, 2)
}
if (componentTag && !componentTag.startsWith(`${name}-`)) {
  fail(
    `[scaffold] --component "${componentTag}" must be prefixed with view name "${name}-" to avoid registry collisions`,
    2,
  )
}

const baseDir = join(repoRoot, 'views', name)
if (existsSync(baseDir)) {
  fail(`[scaffold] directory already exists: ${baseDir}`)
}

mkdirSync(baseDir, { recursive: true })

const title = name.replace(/-/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())

const manifest = `// Route metadata for views/${name}/. See
// docs/ship-adrs/references/docs/view-directory-convention.md.
{
  "name": "${name}",
  "path": "${routePath}",
  "title": "${title}",
  "layout": "default",
  "auth": "public"
}
`

const page = `<state name="ready" value="true" />

<div class="${name}">
  <h1>${title}</h1>
  <p>New view — replace this body.</p>
</div>
`

const controller = `// ${name} view — tier-1 stub. Replace with state, derived, hooks, lifecycle as
// the view grows. See .ship/skills/ship-views/SKILL.md.

export default null
`

const style = `.${name} {
  max-width: 36rem;
  margin: 4rem auto;
  font-family: system-ui, sans-serif;
}
.${name} h1 { margin: 0 0 0.5rem; font-size: 1.5rem; }
`

writeFileSync(join(baseDir, 'manifest.jsonc'), manifest)
writeFileSync(join(baseDir, 'page.sui'), page)
writeFileSync(join(baseDir, 'controller.ts'), controller)
writeFileSync(join(baseDir, 'style.css'), style)

if (componentTag) {
  const tagDir = join(baseDir, '_components', componentTag)
  mkdirSync(tagDir, { recursive: true })
  const componentSui = `<!--
  ${componentTag} — view-private widget for views/${name}/.

  Tag prefix matches the parent view name so the runtime can register it
  without colliding with any other view's _components/ tree.
-->

<prop name="tone" values="default|accent" default="default" reflect />

<style>
  :host { display: block; }
  .root { padding: 1rem; border: 1px solid var(--border, #d0d7de); border-radius: 0.5rem; }
</style>

<div class="root" part="root"><slot></slot></div>
`
  const componentController = `// ${componentTag} — view-private widget controller. Tier-1 stub.

export default null
`
  writeFileSync(join(tagDir, `${componentTag}.sui`), componentSui)
  writeFileSync(join(tagDir, `${componentTag}.controller.ts`), componentController)
}

console.log(`[scaffold] created views/${name}/`)
console.log(`[scaffold]   page.sui  controller.ts  style.css  manifest.jsonc`)
if (componentTag) {
  console.log(`[scaffold]   _components/${componentTag}/${componentTag}.sui (+ controller)`)
}
console.log(`[scaffold] next steps:`)
console.log(`  1. Edit page.sui — replace placeholder body`)
console.log(`  2. Edit manifest.jsonc — set rail/preload/events as needed`)
console.log(`  3. Reload the shell — views auto-mount from .ship/views/`)
