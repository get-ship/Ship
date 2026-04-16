/**
 * Gallery renderer — walks the GROUPS registry and renders a card per
 * sample. Overlay samples get a trigger button + meta caption instead of
 * the raw element so the gallery never pops dialogs / popovers / toasts
 * on load.
 */

import {
  FULL_WIDTH,
  GROUPS,
  SAMPLES,
  allGroupedTags,
  isTrigger,
  type PlainSample,
  type Sample,
  type TriggerSample,
} from './samples'

/** All tags the ship-elements bundle has registered. Discovered at boot. */
async function discoverTags(): Promise<string[]> {
  const res = await fetch('/vendor/ship-elements.js', {
    credentials: 'same-origin',
  })
  const text = await res.text()
  const re = /customElements\.define\(\s*["']([a-z][a-z0-9-]*)["']/g
  const set = new Set<string>()
  let m: RegExpExecArray | null
  while ((m = re.exec(text)) !== null) {
    if (customElements.get(m[1])) set.add(m[1])
  }
  return Array.from(set).sort()
}

function renderPlainBody(tag: string, sample: PlainSample): HTMLElement {
  const wrap = document.createElement('div')
  wrap.className = 'card-render'
  try {
    if (sample.raw && sample.innerHTML) {
      // Raw samples render innerHTML directly into the card-render so
      // multi-element compositions (.sample-triptych, .sample-row) are
      // flex children of .card-render and can claim full width.
      wrap.innerHTML = sample.innerHTML
    } else {
      const el = document.createElement(tag)
      if (sample.props) {
        for (const [k, v] of Object.entries(sample.props)) el.setAttribute(k, v)
      }
      if (sample.innerHTML) el.innerHTML = sample.innerHTML
      wrap.appendChild(el)
    }
  } catch (err) {
    const e = document.createElement('div')
    e.className = 'card-error'
    e.textContent = `render failed: ${(err as Error).message}`
    wrap.appendChild(e)
  }
  return wrap
}

function renderTriggerBody(sample: TriggerSample): HTMLElement {
  const wrap = document.createElement('div')
  wrap.className = 'card-render card-trigger'

  const { overlay, activate, portal } = sample.build()

  if (portal) {
    // Portaled overlays (dialogs, toast container) live on document.body
    // so they sit above the gallery grid. We only append once per card
    // so repeated clicks reuse the same overlay / container.
    if (!overlay.isConnected) document.body.appendChild(overlay)
  } else {
    // Inline overlays (popover / dropdown / tooltip) anchor on the card.
    wrap.appendChild(overlay)
  }

  const btn = document.createElement('ship-button')
  if (sample.buttonTone) btn.setAttribute('tone', sample.buttonTone)
  btn.setAttribute('variant', sample.buttonVariant ?? 'solid')
  btn.setAttribute('size', 'sm')
  btn.textContent = sample.label
  btn.addEventListener('click', () => activate())
  // The tooltip sample wants the trigger button to *be* the tooltip
  // anchor — it's already slotted by the `build` call, so we keep the
  // button in addition as a "focus here to see it" affordance.
  wrap.appendChild(btn)

  return wrap
}

function renderMeta(tag: string, sample: Sample): HTMLElement {
  const meta = document.createElement('div')
  meta.className = 'card-meta'
  const tagLabel = document.createElement('span')
  tagLabel.className = 'card-tag'
  tagLabel.textContent = `<${tag}>`
  meta.appendChild(tagLabel)

  const caption = document.createElement('span')
  caption.className = 'card-props'
  if (isTrigger(sample)) {
    caption.textContent = sample.caption
  } else {
    const attrs = sample.metaAttrs ?? sample.props ?? {}
    const keys = Object.keys(attrs)
    if (keys.length > 0) {
      caption.textContent = keys
        .map((k) => (attrs[k] === '' ? k : `${k}=${attrs[k]}`))
        .join(' · ')
    }
  }
  if (caption.textContent) meta.appendChild(caption)
  return meta
}

function renderCard(tag: string, sample: Sample, full: boolean): HTMLElement {
  const card = document.createElement('div')
  card.className = full ? 'card card-full' : 'card'
  const body = isTrigger(sample)
    ? renderTriggerBody(sample)
    : renderPlainBody(tag, sample)
  card.appendChild(body)
  card.appendChild(renderMeta(tag, sample))
  return card
}

/**
 * Render the gallery into `root`. Returns the count of registered
 * ship-* tags the gallery covered.
 */
export async function renderGalleryAsync(root: HTMLElement): Promise<number> {
  const tags = new Set(await discoverTags())
  root.textContent = ''

  const covered = new Set<string>()

  for (const group of GROUPS) {
    const section = document.createElement('section')
    section.className = 'group'
    const heading = document.createElement('h2')
    heading.textContent = group.title
    section.appendChild(heading)

    const grid = document.createElement('div')
    grid.className = 'group-grid'
    let renderedAny = false

    for (const tag of group.tags) {
      if (!tags.has(tag)) continue
      covered.add(tag)
      const samples = SAMPLES[tag] ?? [{}]
      const full = FULL_WIDTH.has(tag)
      for (const sample of samples) {
        grid.appendChild(renderCard(tag, sample, full))
        renderedAny = true
      }
    }

    if (renderedAny) {
      section.appendChild(grid)
      root.appendChild(section)
    }
  }

  // Catch-all for tags that were registered but not covered by any
  // group. Keeps the gallery self-auditing — new ship-* components
  // appear under "Misc" until they're explicitly grouped. Known child
  // composites (always used inside a parent) are whitelisted out so
  // the "Misc" section only fires when a genuinely new top-level tag
  // is added.
  const grouped = allGroupedTags()
  const orphans = [...tags].filter(
    (t) => !grouped.has(t) && !covered.has(t) && !CHILD_ONLY.has(t),
  )
  if (orphans.length > 0) {
    const section = document.createElement('section')
    section.className = 'group'
    const heading = document.createElement('h2')
    heading.textContent = 'Misc (ungrouped)'
    section.appendChild(heading)
    const grid = document.createElement('div')
    grid.className = 'group-grid'
    for (const tag of orphans.sort()) {
      const samples = SAMPLES[tag] ?? [{}]
      for (const sample of samples) {
        grid.appendChild(renderCard(tag, sample, FULL_WIDTH.has(tag)))
      }
    }
    section.appendChild(grid)
    root.appendChild(section)
  }

  return tags.size
}

/**
 * Child composites that only make sense inside a parent (rendered as
 * part of that parent's sample). Skipped by the "Misc (ungrouped)"
 * catch-all so the gallery stays tidy.
 */
const CHILD_ONLY: ReadonlySet<string> = new Set([
  'ship-breadcrumb-item',
  'ship-git-diff-file',
  'ship-git-diff-hunk',
  'ship-tab',
  'ship-tab-panel',
  'ship-timeline-item',
  'ship-tree-node',
])
