/**
 * Build-time loader over packages/ui/custom-elements.json.
 *
 * Extracts every class declaration with `kind === 'class'` that has a
 * customElement tagName. Returns a typed record per component with
 * everything the docs site needs: tag, summary, attributes, slots,
 * events, parts, CSS custom properties, and the source file.
 *
 * Groups are derived from the source file path:
 *   web-components/              → 'primitives'
 *   web-components/domain/       → 'domain'
 *   web-components/charts/       → 'charts'
 *   web-components/observability/→ 'observability'
 *   web-components/utility/      → 'utility'
 *
 * The shell's old `GROUPS` array separated primitives into topic
 * buckets (Typography, Actions, Forms, Status, Data, Layout, Utility,
 * Overlays). Finer-grained buckets are annotated inline on a per-tag
 * basis here — if a tag isn't listed it stays in `primitives`.
 */

import cem from '../../../../packages/ui/custom-elements.json' with { type: 'json' };

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

// Fine-grained buckets for primitives. Anything not listed stays in
// `primitives` which we surface as "Layout" — CEM-only tags show up
// in the index but don't get a bespoke category.
const CATEGORY_BY_TAG: Record<string, ComponentCategory> = {
  // Typography / heading surface
  'ship-label': 'typography',

  // Actions
  'ship-button': 'actions',
  'ship-toolbar': 'actions',
  'ship-theme-toggle': 'actions',

  // Forms
  'ship-checkbox': 'forms',
  'ship-field': 'forms',
  'ship-file-upload': 'forms',
  'ship-input': 'forms',
  'ship-radio-group': 'forms',
  'ship-select': 'forms',
  'ship-slider': 'forms',
  'ship-switch': 'forms',
  'ship-textarea': 'forms',
  'ship-toggle-group': 'forms',

  // Status
  'ship-alert': 'status',
  'ship-badge': 'status',
  'ship-progress': 'status',
  'ship-spinner': 'status',
  'ship-stat': 'status',
  'ship-status-dot': 'status',

  // Data / lists
  'ship-accordion': 'data',
  'ship-accordion-item': 'data',
  'ship-collapsible': 'data',
  'ship-empty-state': 'data',
  'ship-list-item': 'data',
  'ship-pagination': 'data',
  'ship-tabs': 'data',
  'ship-tab': 'data',
  'ship-tab-panel': 'data',

  // Layout
  'ship-card': 'layout',
  'ship-scroll-area': 'layout',
  'ship-separator': 'layout',
  'ship-navigation-menu': 'layout',

  // Overlays
  'ship-alert-dialog': 'overlays',
  'ship-context-menu': 'overlays',
  'ship-dialog': 'overlays',
  'ship-dropdown-menu': 'overlays',
  'ship-hover-card': 'overlays',
  'ship-popover': 'overlays',
  'ship-tooltip': 'overlays',
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

function deriveCategory(tag: string, path: string): ComponentCategory {
  if (CATEGORY_BY_TAG[tag]) return CATEGORY_BY_TAG[tag];
  if (path.includes('/charts/')) return 'charts';
  if (path.includes('/observability/')) return 'observability';
  if (path.includes('/utility/')) return 'utility';
  if (path.includes('/domain/')) return 'domain';
  return 'layout';
}

function takeFirstSentence(s: string): string {
  const trimmed = s.trim();
  if (!trimmed) return '';
  const idx = trimmed.search(/(?<=\.)\s/);
  return idx > 0 ? trimmed.slice(0, idx) : trimmed.split('\n')[0]!;
}

interface CemDeclaration {
  kind: string;
  name?: string;
  tagName?: string;
  summary?: string;
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
  customElement?: boolean;
}

interface CemModule {
  path?: string;
  declarations?: CemDeclaration[];
}

function toComponent(mod: CemModule, d: CemDeclaration): ComponentDoc | null {
  if (!d.tagName || !d.customElement) return null;
  const path = mod.path ?? '';
  const summary = takeFirstSentence(d.summary ?? d.description ?? '');
  return {
    tag: d.tagName,
    className: d.name ?? d.tagName,
    summary,
    description: d.description ?? '',
    path,
    category: deriveCategory(d.tagName, path),
    attributes: (d.attributes ?? []).map((a) => ({
      name: a.name ?? '',
      description: a.description,
      type: a.type?.text,
      default: a.default,
      fieldName: a.fieldName,
    })),
    slots: (d.slots ?? []).map((s) => ({
      name: s.name ?? '(default)',
      description: s.description,
    })),
    events: (d.events ?? []).map((e) => ({
      name: e.name ?? '',
      description: e.description,
      type: e.type?.text,
    })),
    parts: (d.cssParts ?? []).map((p) => ({
      name: p.name ?? '',
      description: p.description,
    })),
    cssProperties: (d.cssProperties ?? []).map((c) => ({
      name: c.name ?? '',
      description: c.description,
      default: c.default,
      syntax: c.syntax,
    })),
  };
}

const raw = cem as { modules?: CemModule[] };

export const COMPONENTS: ComponentDoc[] = (raw.modules ?? [])
  .flatMap((m) =>
    (m.declarations ?? []).map((d) => toComponent(m, d)).filter((x): x is ComponentDoc => x !== null),
  )
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
