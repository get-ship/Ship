#!/usr/bin/env node
// Generate a token-efficient element catalog from custom-elements.json.
// One compact line per element. Grouped by subdir. Used by the
// ship-components skill so agents get a scannable index in-context
// without loading the full CEM.

import fs from 'node:fs'
import path from 'node:path'

const root = path.resolve(new URL(import.meta.url).pathname, '../..')
const cem = JSON.parse(fs.readFileSync(path.join(root, 'custom-elements.json'), 'utf8'))

const groups = {
  core: [],
  domain: [],
  observability: [],
  charts: [],
  utility: [],
}

function groupFor(modulePath) {
  if (modulePath.includes('/domain/')) return 'domain'
  if (modulePath.includes('/observability/')) return 'observability'
  if (modulePath.includes('/charts/')) return 'charts'
  if (modulePath.includes('/utility/')) return 'utility'
  return 'core'
}

for (const mod of cem.modules) {
  for (const d of mod.declarations || []) {
    if (!d.customElement || !d.tagName) continue
    const attrs = (d.attributes || []).map(a => {
      const name = a.name || a.fieldName
      const t = a.type?.text
      // Enum-like? Pull values if type is a union of string literals.
      if (t && t.includes("'") && t.includes('|')) {
        const values = [...t.matchAll(/'([^']+)'/g)].map(m => m[1])
        return `${name}=${values.join('|')}`
      }
      if (t === 'boolean') return name
      return `${name}`
    })
    const slots = (d.slots || []).map(s => s.name || '(default)')
    const events = (d.events || []).map(e => e.name)
    groups[groupFor(mod.path)].push({
      tag: d.tagName,
      attrs,
      slots,
      events,
    })
  }
}

const lines = []
lines.push('# Ship element catalog\n')
lines.push('Compact reference auto-generated from `custom-elements.json`. For depth (types, defaults, descriptions, CSS custom properties), load the full CEM at `/api/assets/custom-elements.json`.\n')
lines.push('Convention: `attr=value|value` = enum, bare `attr` = boolean.\n')

const headings = {
  core: 'Core primitives',
  domain: 'Pipeline domain',
  observability: 'Observability',
  charts: 'Charts',
  utility: 'Utility',
}

for (const key of ['core', 'domain', 'observability', 'charts', 'utility']) {
  const arr = groups[key].sort((a, b) => a.tag.localeCompare(b.tag))
  if (!arr.length) continue
  lines.push(`\n## ${headings[key]}\n`)
  for (const el of arr) {
    const attrPart = el.attrs.length ? ` [${el.attrs.join(' ')}]` : ''
    const slotPart = el.slots.length ? ` slots: ${el.slots.join(', ')}` : ''
    const evtPart = el.events.length ? ` events: ${el.events.join(', ')}` : ''
    lines.push(`- \`<${el.tag}>\`${attrPart}${slotPart}${evtPart}`)
  }
}

const out = lines.join('\n') + '\n'
fs.writeFileSync(path.join(root, 'catalog.md'), out)
console.log(`catalog.md: ${out.length} bytes, ${Object.values(groups).reduce((n, g) => n + g.length, 0)} elements`)
