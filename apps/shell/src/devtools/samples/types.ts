/**
 * Shared types + small helpers for the devtools gallery samples.
 *
 * Samples are split across small topic files (typography, forms, …) and
 * merged in `./index.ts`. Keeping this file lean means each topic file
 * imports only what it actually needs.
 */

export type PlainSample = {
  props?: Record<string, string>
  innerHTML?: string
  /**
   * When set, `innerHTML` is rendered directly into `.card-render` with no
   * wrapping `<${tag}>`. Needed for samples that compose multiple top-level
   * elements (`.sample-triptych` etc). The tag in the meta row still names
   * the primary component being showcased.
   */
  raw?: boolean
  /** Attributes to display in the meta line; defaults to `props`. */
  metaAttrs?: Record<string, string>
}

export type TriggerSample = {
  /** Label shown on the trigger button. */
  label: string
  /** Button tone / variant for the trigger itself. */
  buttonTone?: string
  buttonVariant?: string
  /**
   * Build the overlay element. The caller appends it to the card and
   * wires up `activate` to fire when the trigger button is clicked.
   * If the overlay should be portaled (dialog / alert-dialog / toast
   * container) set `portal: true` — the element is appended to document
   * body instead of the card so it renders above everything else.
   */
  build: () => {
    overlay: HTMLElement
    activate: () => void
    portal?: boolean
  }
  /** Caption shown in the meta line instead of props. */
  caption: string
}

export type Sample = PlainSample | TriggerSample

export function isTrigger(s: Sample): s is TriggerSample {
  return typeof (s as TriggerSample).build === 'function'
}

export type Group = { title: string; tags: string[] }

/**
 * High-variance primitives render one big card with a tone × variant
 * matrix; compact primitives keep the default grid slot.
 */
export const FULL_WIDTH: ReadonlySet<string> = new Set([
  'ship-button',
  'ship-badge',
  'ship-chip',
  'ship-alert',
  'ship-input',
  'ship-select',
  'ship-textarea',
  'ship-switch',
  'ship-checkbox',
  'ship-file-upload',
  'ship-tabs',
  'ship-table',
  'ship-pipeline-dag',
  'ship-git-diff',
  'ship-timeline',
  'ship-tree-view',
  'ship-toast',
  'ship-empty-state',
  'ship-bar-chart',
  'ship-line-chart',
  'ship-area-chart',
  'ship-heatmap',
  'ship-meter',
  'ship-code',
  'ship-markdown',
  'ship-json-view',
  'ship-diff',
  'ship-pipeline-definition',
  'ship-pipeline-run',
  'ship-toolbar',
  'ship-breadcrumb',
  'ship-accordion',
  'ship-radio-group',
  'ship-toggle-group',
  'ship-navigation-menu',
  'ship-slider',
  'ship-field',
  'ship-history-row',
  'ship-stat',
  'ship-log-line',
  'ship-event-row',
  'ship-card',
  'ship-skeleton',
  'ship-spec-row',
  'ship-pagination',
])

// Shared helpers

export const TONES = [
  'default',
  'info',
  'success',
  'warning',
  'destructive',
  'accent',
] as const

export const VARIANTS_BTN = ['solid', 'soft', 'outline', 'ghost'] as const
export const VARIANTS_BADGE = ['solid', 'soft', 'outline'] as const

/** One-line wrapper that lets the card-render flex-wrap children.
 * `raw: true` so the sample-row is a direct flex child of .card-render
 * and can grow to fill the card width. */
export function row(children: string): PlainSample {
  return {
    raw: true,
    innerHTML: `<div class="sample-row">${children}</div>`,
  }
}

/** tone × variant grid for high-variance primitives.
 * Renders as a labeled grid: one row per tone, with a leading micro-label
 * naming the tone so the matrix reads like a spec sheet rather than a
 * wall of identical words. */
export function matrix(
  tag: string,
  tones: readonly string[],
  variants: readonly string[],
  label: string,
): PlainSample {
  // Header row of variant names.
  const header =
    `<span class="matrix-h"></span>` +
    variants.map((v) => `<span class="matrix-h">${v}</span>`).join('')

  const rows = tones
    .map((tone) => {
      const cells = variants
        .map(
          (v) => `<${tag} tone="${tone}" variant="${v}">${tone}</${tag}>`,
        )
        .join('')
      return `<span class="matrix-h">${tone}</span>${cells}`
    })
    .join('')

  return {
    raw: true,
    innerHTML: `<div class="sample-matrix" data-cols="${variants.length + 1}">${header}${rows}</div>`,
    metaAttrs: {
      tone: tones.join('|'),
      variant: variants.join('|'),
      note: label,
    },
  }
}

// Group registry — order matters, renders top to bottom.

export const GROUPS: Group[] = [
  {
    title: 'Typography & Text',
    tags: [
      'ship-markdown',
      'ship-code',
      'ship-kbd',
      'ship-link',
      'ship-timestamp',
      'ship-json-view',
    ],
  },
  {
    title: 'Buttons & Actions',
    tags: ['ship-button', 'ship-chip', 'ship-copy-button'],
  },
  {
    title: 'Forms',
    tags: [
      'ship-input',
      'ship-textarea',
      'ship-select',
      'ship-checkbox',
      'ship-switch',
      'ship-radio-group',
      'ship-slider',
      'ship-toggle-group',
      'ship-file-upload',
      'ship-field',
      'ship-label',
    ],
  },
  {
    title: 'Status & Feedback',
    tags: [
      'ship-badge',
      'ship-status-dot',
      'ship-spinner',
      'ship-progress',
      'ship-progress-ring',
      'ship-alert',
    ],
  },
  {
    title: 'Data Display',
    tags: [
      'ship-stat',
      'ship-table',
      'ship-meter',
      'ship-gauge',
      'ship-donut',
      'ship-sparkline',
      'ship-bar-chart',
      'ship-line-chart',
      'ship-area-chart',
      'ship-heatmap',
      'ship-pagination',
    ],
  },
  {
    title: 'Layout',
    tags: [
      'ship-card',
      'ship-separator',
      'ship-toolbar',
      'ship-breadcrumb',
      'ship-tabs',
      'ship-accordion',
      'ship-collapsible',
      'ship-scroll-area',
      'ship-navigation-menu',
      'ship-tree-view',
      'ship-timeline',
      'ship-list-item',
    ],
  },
  {
    title: 'Utility',
    tags: [
      'ship-icon',
      'ship-avatar',
      'ship-theme-toggle',
      'ship-metachip',
      'ship-skeleton',
      'ship-step-segments',
    ],
  },
  {
    title: 'Empty states',
    tags: ['ship-empty-state'],
  },
  {
    title: 'Overlays',
    tags: [
      'ship-dialog',
      'ship-alert-dialog',
      'ship-popover',
      'ship-tooltip',
      'ship-dropdown-menu',
      'ship-context-menu',
      'ship-hover-card',
      'ship-toast-container',
      'ship-toast',
    ],
  },
  {
    title: 'Domain',
    tags: [
      'ship-pipeline-dag',
      'ship-pipeline-step',
      'ship-pipeline-run',
      'ship-pipeline-definition',
      'ship-spec-row',
      'ship-history-row',
      'ship-event-row',
      'ship-log-line',
    ],
  },
  {
    title: 'Git diff',
    tags: ['ship-git-diff', 'ship-git-diff-line', 'ship-diff'],
  },
  {
    title: 'Annotations',
    tags: [
      'ship-annotation-layer',
      'ship-annotation-marker',
      'ship-annotation-list',
    ],
  },
]

export function allGroupedTags(): Set<string> {
  const out = new Set<string>()
  for (const g of GROUPS) for (const t of g.tags) out.add(t)
  return out
}
