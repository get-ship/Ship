// Prompt construction for the eval rig. The model receives:
//   1) a system primer describing .html.set syntax + the SDK type sketch
//   2) the primitive catalog (so it knows which <ship-*> elements exist)
//   3) two few-shot examples (a static view + a stateful counter)
//   4) the task feature description and any starter fixtures
//
// The output contract is strict: a fenced ```html.set``` block, optionally
// followed by a fenced ```ts``` block. The dispatcher parses both back out.

import type { PrimitiveCatalog, PrimitiveDef } from './catalog'

export interface PromptInputs {
  description: string
  catalog: PrimitiveCatalog
  fixtures: { template: string; setup?: string }
}

export interface PromptParts {
  system: string
  user: string
}

const SYSTEM_PRIMER = `You author Ship .html.set views. A view is plain HTML augmented with three top-level directives and a small expression grammar.

Top-level directives:
  <data name="x" source="ship.foo.bar()" [live] />   — fetches data; live makes it a streaming signal.
  <state name="x" value="0" />                       — local reactive value, written via x.set(...).
  <component name="x-tag"> ... </component>          — turns the file into a custom element.

Template grammar:
  {expr}                                — text-position expression.
  attr={expr}                           — dynamic attribute.
  on:event={(e) => ...}                 — event listener.
  bind:value={signalName}               — two-way binding (input-like elements only).
  {#if expr} ... {:else} ... {/if}      — conditional.
  {#for item of expr key={item.id}} ... {/for} — keyed list.
  <while-loading bind="name"> ... </while-loading>   — render while a <data> is pending.
  <while-error bind="name" let:err> ... </while-error> — render when a <data> errors.

Reactive values are read by calling them: \`count()\` reads, \`count.set(n)\` writes.
A \`<data live>\` declaration exposes a Result-shaped signal; check \`x().kind === 'ok'\` then read \`x().value\`.

Setup file (optional, paired .ts):
  export default function setup({ ship, route, navigate }) {
    return { /* extras merged into template scope */ }
  }

Hard rules — violating any of these fails the eval:
  - No raw <script> tags. No eval. No new Function(). No document.write.
  - No customElements.define — that's primitive territory, not view territory.
  - Use <ship-*> primitives from the catalog rather than reinventing buttons or inputs.
  - Output exactly one fenced \`\`\`html.set\`\`\` block. If a setup file is needed, append one fenced \`\`\`ts\`\`\` block. No prose outside the blocks.`

const FEW_SHOT_STATIC = {
  description: 'A static info card listing the three Ship core values.',
  template: `<section class="card">
  <h2>Ship core values</h2>
  <ul>
    <li>Boring infra, novel surface.</li>
    <li>Server-rendered first.</li>
    <li>Agent-writable by construction.</li>
  </ul>
</section>`,
}

const FEW_SHOT_COUNTER = {
  description: 'A counter view: shows a number and two buttons that increment and reset it.',
  template: `<state name="count" value="0" />

<div class="counter">
  <p>count: <span data-test="value">{count()}</span></p>
  <ship-button data-test="inc" on:click={() => count.set(count() + 1)}>increment</ship-button>
  <ship-button data-test="reset" tone="neutral" on:click={() => count.set(0)}>reset</ship-button>
</div>`,
}

/** Render a primitive catalog as a compact, model-readable summary. */
export function renderCatalog(catalog: PrimitiveCatalog): string {
  return catalog.primitives.map(renderPrimitive).join('\n\n')
}

function renderPrimitive(p: PrimitiveDef): string {
  const lines: string[] = [`<${p.tag}>${p.summary ? ' — ' + p.summary : ''}`]
  if (p.props.length) {
    lines.push('  props:')
    for (const prop of p.props) {
      const t =
        prop.type === 'union' && prop.values
          ? prop.values.join('|')
          : prop.type
      const def = prop.default !== undefined ? ` default=${JSON.stringify(prop.default)}` : ''
      const bind = prop.bindable ? ' bindable' : ''
      lines.push(`    - ${prop.name}: ${t}${def}${bind}`)
    }
  }
  if (p.slots.length) {
    lines.push('  slots:')
    for (const slot of p.slots) {
      lines.push(`    - ${slot.name}${slot.summary ? ': ' + slot.summary : ''}`)
    }
  }
  if (p.events.length) {
    lines.push('  events:')
    for (const ev of p.events) {
      lines.push(`    - ${ev.name}${ev.summary ? ': ' + ev.summary : ''}`)
    }
  }
  return lines.join('\n')
}

function renderFewShot(ex: { description: string; template: string }): string {
  return `Example feature: ${ex.description}\n\nExample output:\n\n\`\`\`html.set\n${ex.template}\n\`\`\``
}

/** Assemble the system + user messages for one task. */
export function buildPrompt(input: PromptInputs): PromptParts {
  const catalogBlock = renderCatalog(input.catalog)
  const fewShot = [renderFewShot(FEW_SHOT_STATIC), renderFewShot(FEW_SHOT_COUNTER)].join(
    '\n\n---\n\n',
  )

  const fixtureBlock =
    input.fixtures.template.trim() || input.fixtures.setup?.trim()
      ? `\n\nStarter fixture (extend or replace as needed):\n\n\`\`\`html.set\n${input.fixtures.template}\n\`\`\`${
          input.fixtures.setup
            ? `\n\n\`\`\`ts\n${input.fixtures.setup}\n\`\`\``
            : ''
        }`
      : ''

  const user = `Primitive catalog (${input.catalog.version}):

${catalogBlock}

---

${fewShot}

---

Feature to implement:

${input.description.trim()}${fixtureBlock}

Reply with one \`\`\`html.set\`\`\` block, optionally followed by one \`\`\`ts\`\`\` block. No prose.`

  return { system: SYSTEM_PRIMER, user }
}

/** Pull the .html.set + optional .ts block out of a model reply. */
export function parseModelReply(raw: string): { template: string; setup?: string } {
  const html = matchFence(raw, 'html.set') ?? matchFence(raw, 'html')
  if (!html) {
    throw new Error('model reply did not contain an ```html.set``` block')
  }
  const ts = matchFence(raw, 'ts') ?? matchFence(raw, 'typescript')
  return ts ? { template: html, setup: ts } : { template: html }
}

function matchFence(raw: string, tag: string): string | undefined {
  // Match a fenced block whose info-string starts with `tag`. We
  // tolerate trailing junk on the info line so ```html.set view``` works.
  const re = new RegExp(
    '```' + escapeRegex(tag) + '[^\\n]*\\n([\\s\\S]*?)\\n?```',
    'm',
  )
  const m = re.exec(raw)
  return m && m[1] !== undefined ? m[1] : undefined
}

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}
