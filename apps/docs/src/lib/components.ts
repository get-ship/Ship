/**
 * Build-time loader over the workspace SUI catalog.
 *
 * Reads `.deck/catalogs/workspace.json` — the canonical catalog
 * produced by `@ship/sui-catalog` (CLI: `bun packages/deck/catalog/src/cli.ts`)
 * which walks every workspace package's `customElements` manifest plus
 * the per-component CEMs in `.ship/sui-out/`. Emits the typed record per
 * `<sui-*>` primitive that the docs site renders.
 */

import catalog from '../../../../.deck/catalogs/workspace.json' with { type: 'json' };

export type ComponentCategory =
  | 'typography'
  | 'actions'
  | 'forms'
  | 'status'
  | 'data'
  | 'layout'
  | 'overlays'
  | 'utility'
  | 'charts'
  | 'observability'
  | 'domain';

export interface ComponentAttribute {
  name: string;
  description?: string;
  type?: string;
  default?: string;
  fieldName?: string;
}

export interface ComponentSlot {
  name: string;
  description?: string;
}

export interface ComponentEvent {
  name: string;
  description?: string;
  type?: string;
}

export interface ComponentPart {
  name: string;
  description?: string;
}

export interface ComponentCssProp {
  name: string;
  description?: string;
  default?: string;
  syntax?: string;
}

export interface ComponentDoc {
  tag: string;
  className: string;
  summary: string;
  description: string;
  category: ComponentCategory;
  path: string;
  attributes: ComponentAttribute[];
  slots: ComponentSlot[];
  events: ComponentEvent[];
  parts: ComponentPart[];
  cssProperties: ComponentCssProp[];
}

const CATEGORY_BY_TAG: Record<string, ComponentCategory> = {
  // Typography
  'sui-label': 'typography',

  // Actions
  'sui-button': 'actions',
  'sui-toolbar': 'actions',
  'sui-theme-toggle': 'actions',
  'sui-theme-picker': 'actions',

  // Forms
  'sui-checkbox': 'forms',
  'sui-field': 'forms',
  'sui-file-upload': 'forms',
  'sui-input': 'forms',
  'sui-radio-group': 'forms',
  'sui-radio': 'forms',
  'sui-select': 'forms',
  'sui-slider': 'forms',
  'sui-switch': 'forms',
  'sui-textarea': 'forms',
  'sui-toggle-group': 'forms',
  'sui-toggle': 'forms',

  // Status
  'sui-alert': 'status',
  'sui-badge': 'status',
  'sui-progress': 'status',
  'sui-progress-ring': 'status',
  'sui-spinner': 'status',
  'sui-stat': 'status',
  'sui-status-dot': 'status',
  'sui-loading': 'status',
  'sui-error-state': 'status',
  'sui-status-bar': 'status',

  // Data / lists
  'sui-accordion': 'data',
  'sui-accordion-item': 'data',
  'sui-collapsible': 'data',
  'sui-empty-state': 'data',
  'sui-list-tile': 'data',
  'sui-pagination': 'data',
  'sui-tabs': 'data',
  'sui-tab': 'data',
  'sui-tab-panel': 'data',
  'sui-stepper': 'data',

  // Layout
  'sui-card': 'layout',
  'sui-scroll-area': 'layout',
  'sui-separator': 'layout',
  'sui-divider': 'layout',
  'sui-navigation-menu': 'layout',
  'sui-view-chrome': 'layout',
  'sui-portal': 'layout',
  'sui-error-boundary': 'layout',

  // Overlays
  'sui-alert-dialog': 'overlays',
  'sui-context-menu': 'overlays',
  'sui-dialog': 'overlays',
  'sui-drawer': 'overlays',
  'sui-dropdown-menu': 'overlays',
  'sui-hover-card': 'overlays',
  'sui-popover': 'overlays',
  'sui-tooltip': 'overlays',
  'sui-edit-overlay': 'overlays',

  // Utility
  'sui-avatar': 'utility',
  'sui-breadcrumb': 'utility',
  'sui-breadcrumb-item': 'utility',
  'sui-chip': 'utility',
  'sui-icon': 'utility',
  'sui-kbd': 'utility',
  'sui-link': 'utility',
  'sui-metachip': 'utility',
  'sui-skeleton': 'utility',
  'sui-step-segments': 'utility',
  'sui-toast': 'utility',
  'sui-toast-container': 'utility',
  'sui-toast-host': 'utility',

  // Charts
  'sui-area-chart': 'charts',
  'sui-bar-chart': 'charts',
  'sui-line-chart': 'charts',
  'sui-sparkline': 'charts',
  'sui-donut': 'charts',
  'sui-gauge': 'charts',
  'sui-meter': 'charts',
  'sui-heatmap': 'charts',

  // Observability
  'sui-code': 'observability',
  'sui-copy-button': 'observability',
  'sui-diff': 'observability',
  'sui-git-diff': 'observability',
  'sui-history-row': 'observability',
  'sui-json-view': 'observability',
  'sui-log-stream': 'observability',
  'sui-markdown': 'observability',
  'sui-table': 'observability',
  'sui-timeline': 'observability',
  'sui-timeline-item': 'observability',
  'sui-timestamp': 'observability',
  'sui-tree-view': 'observability',
  'sui-annotation-layer': 'observability',
  'sui-annotation-marker': 'observability',
  'sui-annotation-list': 'observability',

  // Domain
  'sui-auth-dialog': 'domain',
  'sui-pipeline-dag': 'domain',
  'sui-pipeline-step': 'domain',
  'sui-pipeline-step-def': 'domain',
  'sui-pipeline-definition': 'domain',
  'sui-pipeline-run': 'domain',
  'sui-graph-canvas': 'domain',
  'sui-harness-blob': 'domain',
};

export const CATEGORY_LABELS: Record<ComponentCategory, string> = {
  typography: 'Typography',
  actions: 'Actions',
  forms: 'Forms',
  status: 'Status',
  data: 'Data',
  layout: 'Layout',
  overlays: 'Overlays',
  utility: 'Utility',
  charts: 'Charts',
  observability: 'Observability',
  domain: 'Domain',
};

export const CATEGORY_ORDER: ComponentCategory[] = [
  'typography',
  'actions',
  'forms',
  'status',
  'data',
  'layout',
  'overlays',
  'utility',
  'charts',
  'observability',
  'domain',
];

function deriveCategory(tag: string): ComponentCategory {
  return CATEGORY_BY_TAG[tag] ?? 'layout';
}

/**
 * Tags that opt into their own bundle via the `<bundle separate />`
 * directive on their `.sui` source — bundled separately and loaded
 * on-demand by the shell (not in the main `sui-components.js` bundle
 * the docs site links from the layout).
 * Excluded from the per-component pages because their custom elements
 * are not registered at the URL these docs serve from, so any preview
 * would render an undefined-element husk.
 */
const ISLAND_TAGS = new Set([
  'sui-code-editor',
  'sui-command-bar',
  'sui-command-chat',
  'sui-devtools-components',
  'sui-devtools-schema',
  'sui-devtools-signals',
  'sui-devtools-source',
  'sui-launcher',
  'sui-markdown-editor',
  'sui-nav-top',
  'sui-panel-dock',
  'sui-rail',
  'sui-rail-item',
  'sui-rail-section',
  'sui-terminal',
]);

/**
 * Tags that are deprecated or strictly internal — excluded from the
 * docs inventory so the gallery doesn't promote them. `sui-theme-toggle`
 * is superseded by `<sui-theme-picker>` (full brand palette); the
 * sub-element composites only render inside their parent and have a
 * shot on the parent's page.
 */
const HIDDEN_TAGS = new Set([
  'sui-theme-toggle',
  // Composite-child elements that only render meaningfully inside a
  // specific parent — keep them documented inside the parent's preview
  // rather than as standalone pages where they'd render an empty
  // husk or duplicate the parent's example verbatim.
  'sui-tab',
  'sui-tab-panel',
  'sui-tree-item',
  'sui-accordion-item',
  'sui-breadcrumb-item',
  'sui-navigation-item',
  'sui-pipeline-step-def',
  'sui-timeline-item',
  'sui-git-diff-file',
  'sui-git-diff-hunk',
  'sui-git-diff-line',
  'sui-toast-host',
  'sui-annotation-layer',
  'sui-annotation-marker',
  'sui-annotation-list',
  // Shell chrome — only meaningful inside the Ship Studio shell.
  // Documenting them on /components/ promotes infrastructure that
  // customers shouldn't compose into their own apps. The shell
  // catalog (apps/shell/) imports them directly.
  'sui-launcher',
  'sui-nav-top',
  'sui-panel-dock',
  'sui-rail',
  'sui-rail-item',
  'sui-rail-section',
  'sui-view-chrome',
  'sui-view-picker',
  'sui-brand-mark',
  'sui-devtools-components',
  'sui-devtools-schema',
  'sui-devtools-signals',
  'sui-devtools-source',
  // View-authoring overlay components — only meaningful when shell's
  // edit mode is active. Same reasoning as devtools-*.
  'sui-va-activity',
  'sui-va-annotation-pins',
  'sui-va-compile',
  'sui-va-components',
  'sui-va-source',
  // Examples-only.
  'sui-app',
]);

function takeFirstSentence(s: string): string {
  const trimmed = s.trim();
  if (!trimmed) return '';
  const idx = trimmed.search(/(?<=\.)\s/);
  return idx > 0 ? trimmed.slice(0, idx) : trimmed.split('\n')[0]!;
}

interface CatalogEntry {
  tagName: string;
  sourcePath: string;
  description?: string;
  attributes?: Array<{
    name?: string;
    description?: string;
    type?: { text?: string };
    default?: string;
    fieldName?: string;
  }>;
  slots?: Array<{ name?: string; description?: string }>;
  events?: Array<{ name?: string; description?: string; type?: { text?: string } }>;
  cssParts?: Array<{ name?: string; description?: string }>;
  cssProperties?: Array<{
    name?: string;
    description?: string;
    default?: string;
    syntax?: string;
  }>;
}

function toComponent(e: CatalogEntry): ComponentDoc | null {
  if (!e.tagName) return null;
  if (ISLAND_TAGS.has(e.tagName)) return null;
  if (HIDDEN_TAGS.has(e.tagName)) return null;
  const description = e.description ?? '';
  const summary = takeFirstSentence(description);
  return {
    tag: e.tagName,
    className: e.tagName,
    summary,
    description,
    path: e.sourcePath ?? '',
    category: deriveCategory(e.tagName),
    attributes: (e.attributes ?? []).map((a) => ({
      name: a.name ?? '',
      description: a.description,
      type: a.type?.text,
      default: a.default,
      fieldName: a.fieldName,
    })),
    slots: (e.slots ?? []).map((s) => ({
      name: s.name ?? '(default)',
      description: s.description,
    })),
    events: (e.events ?? []).map((ev) => ({
      name: ev.name ?? '',
      description: ev.description,
      type: ev.type?.text,
    })),
    parts: (e.cssParts ?? []).map((p) => ({
      name: p.name ?? '',
      description: p.description,
    })),
    cssProperties: (e.cssProperties ?? []).map((c) => ({
      name: c.name ?? '',
      description: c.description,
      default: c.default,
      syntax: c.syntax,
    })),
  };
}

const raw = catalog as { entries?: CatalogEntry[] };

export const COMPONENTS: ComponentDoc[] = (raw.entries ?? [])
  .map(toComponent)
  .filter((x): x is ComponentDoc => x !== null)
  .sort((a, b) => a.tag.localeCompare(b.tag));

export const COMPONENTS_BY_TAG: Record<string, ComponentDoc> = Object.fromEntries(
  COMPONENTS.map((c) => [c.tag, c] as const),
);

export function componentsByCategory(): Record<ComponentCategory, ComponentDoc[]> {
  const buckets = Object.fromEntries(
    CATEGORY_ORDER.map((c) => [c, [] as ComponentDoc[]]),
  ) as Record<ComponentCategory, ComponentDoc[]>;
  for (const c of COMPONENTS) buckets[c.category].push(c);
  return buckets;
}
