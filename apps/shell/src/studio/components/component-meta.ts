// Components-section metadata — the editorial layer over the live
// catalog (`.ship/sui-out/catalogs/published.json`, served at
// `/api/sui/catalog/published`). The catalog is the source of truth for
// prop schemas, but its `description` and `slots` fields are currently
// empty for every primitive (a CEM-generation gap in the primitives
// crate — flagged in the job commit). Until those land, this module
// supplies the grouped-index categories and one-line blurbs the design
// (.artifacts/studio-design/components-list.html) calls for.
//
// One source of truth: category + blurb live together here, keyed by
// tag. `catalog.ts` joins this against the live prop schemas; nothing
// else hardcodes the component list. Tags present in the catalog but
// absent here fall into the trailing "Other" group so a newly-published
// primitive is never silently dropped.

export type CategoryId =
  | 'actions' | 'inputs' | 'layout' | 'feedback'
  | 'indicators' | 'overlays' | 'data' | 'charts' | 'other'

export interface Category {
  id: CategoryId
  label: string
  blurb: string
}

// Display order of the grouped index. "Other" is appended by catalog.ts
// only when it has members, so it never shows an empty header.
export const CATEGORIES: ReadonlyArray<Category> = [
  { id: 'actions',    label: 'Actions',    blurb: 'user-triggered changes; buttons, toolbars, segmented controls' },
  { id: 'inputs',     label: 'Inputs',     blurb: 'form fields and selection controls' },
  { id: 'layout',     label: 'Layout',     blurb: 'structural containers and dividers' },
  { id: 'feedback',   label: 'Feedback',   blurb: 'status, loading, errors, transient messages' },
  { id: 'indicators', label: 'Indicators', blurb: 'small visual tags and badges' },
  { id: 'overlays',   label: 'Overlays',   blurb: 'modals, popovers, menus, tabs, disclosure' },
  { id: 'data',       label: 'Data',       blurb: 'tables, lists, json, diffs, timelines' },
  { id: 'charts',     label: 'Charts',     blurb: 'data visualization primitives' },
  { id: 'other',      label: 'Other',      blurb: 'uncategorized published components' },
]

export interface TagMeta { category: CategoryId; blurb: string }

// Tag → (category, one-line blurb). Curated editorial copy in the same
// spirit as themes/theme-registry.ts. Keep blurbs terse — they render as
// a single ellipsized row in the grouped index.
export const TAG_META: Record<string, TagMeta> = {
  // ── Actions ──────────────────────────────────────────────────────
  'sui-button':           { category: 'actions', blurb: 'primary action surface' },
  'sui-toggle':           { category: 'actions', blurb: 'two-state pressable' },
  'sui-toggle-group':     { category: 'actions', blurb: 'multi-select toolbar' },
  'sui-segmented':        { category: 'actions', blurb: 'multi-option pill' },
  'sui-segmented-option': { category: 'actions', blurb: 'segmented control choice' },
  'sui-toolbar':          { category: 'actions', blurb: 'action group container' },
  'sui-theme-toggle':     { category: 'actions', blurb: 'light/dark theme switch' },
  'sui-copy-button':      { category: 'actions', blurb: 'copy-to-clipboard affordance' },
  'sui-link':             { category: 'actions', blurb: 'navigational anchor' },
  'sui-stepper':          { category: 'actions', blurb: 'numeric increment control' },
  'sui-step-segments':    { category: 'actions', blurb: 'multi-step progress segments' },
  'sui-pagination':       { category: 'actions', blurb: 'page navigation control' },

  // ── Inputs ───────────────────────────────────────────────────────
  'sui-input':       { category: 'inputs', blurb: 'single-line text field' },
  'sui-textarea':    { category: 'inputs', blurb: 'multi-line text field' },
  'sui-select':      { category: 'inputs', blurb: 'native dropdown' },
  'sui-combobox':    { category: 'inputs', blurb: 'filterable select' },
  'sui-autocomplete':{ category: 'inputs', blurb: 'typeahead suggestions' },
  'sui-checkbox':    { category: 'inputs', blurb: 'boolean toggle' },
  'sui-radio':       { category: 'inputs', blurb: 'single radio option' },
  'sui-radio-group': { category: 'inputs', blurb: 'single-option radio set' },
  'sui-switch':      { category: 'inputs', blurb: 'on/off toggle' },
  'sui-slider':      { category: 'inputs', blurb: 'numeric range' },
  'sui-date-picker': { category: 'inputs', blurb: 'calendar date selection' },
  'sui-color-picker':{ category: 'inputs', blurb: 'color value selection' },
  'sui-file-upload': { category: 'inputs', blurb: 'file picker / drop zone' },
  'sui-tag-input':   { category: 'inputs', blurb: 'token / chip entry' },
  'sui-field':       { category: 'inputs', blurb: 'label + input + hint composer' },

  // ── Layout ───────────────────────────────────────────────────────
  'sui-card':        { category: 'layout', blurb: 'elevated content container' },
  'sui-stack':       { category: 'layout', blurb: 'vertical flow with gap' },
  'sui-cluster':     { category: 'layout', blurb: 'horizontal wrap with gap' },
  'sui-center':      { category: 'layout', blurb: 'centering box' },
  'sui-grid':        { category: 'layout', blurb: 'responsive grid' },
  'sui-divider':     { category: 'layout', blurb: 'horizontal rule with label slot' },
  'sui-separator':   { category: 'layout', blurb: 'thin rule' },
  'sui-scroll-area': { category: 'layout', blurb: 'styled scroll container' },
  'sui-portal':      { category: 'layout', blurb: 'render-to-body teleport' },

  // ── Feedback ─────────────────────────────────────────────────────
  'sui-alert':          { category: 'feedback', blurb: 'inline tone-coded message' },
  'sui-toast':          { category: 'feedback', blurb: 'transient overlay message' },
  'sui-toast-container':{ category: 'feedback', blurb: 'toast stack region' },
  'sui-toast-host':     { category: 'feedback', blurb: 'toast mount point' },
  'sui-empty-state':    { category: 'feedback', blurb: 'no-content scaffold' },
  'sui-error-state':    { category: 'feedback', blurb: 'error scaffold' },
  'sui-error-boundary': { category: 'feedback', blurb: 'render-error catch surface' },
  'sui-skeleton':       { category: 'feedback', blurb: 'loading placeholder shapes' },
  'sui-spinner':        { category: 'feedback', blurb: 'indeterminate loading' },
  'sui-loading':        { category: 'feedback', blurb: 'loading overlay' },
  'sui-progress':       { category: 'feedback', blurb: 'linear progress bar' },
  'sui-progress-ring':  { category: 'feedback', blurb: 'radial progress' },
  'sui-meter':          { category: 'feedback', blurb: 'bounded measurement gauge' },

  // ── Indicators ───────────────────────────────────────────────────
  'sui-badge':      { category: 'indicators', blurb: 'tone-coded label' },
  'sui-chip':       { category: 'indicators', blurb: 'interactive tag with slots' },
  'sui-status-dot': { category: 'indicators', blurb: 'tiny tone-coded dot' },
  'sui-avatar':     { category: 'indicators', blurb: 'user / identity tile' },
  'sui-app-icon':   { category: 'indicators', blurb: 'app / surface mark' },
  'sui-icon':       { category: 'indicators', blurb: 'lucide glyph' },
  'sui-kbd':        { category: 'indicators', blurb: 'keyboard key cap' },
  'sui-label':      { category: 'indicators', blurb: 'form / field label' },
  'sui-metachip':   { category: 'indicators', blurb: 'metadata key-value chip' },
  'sui-stat':       { category: 'indicators', blurb: 'number + label tile' },
  'sui-timestamp':  { category: 'indicators', blurb: 'relative / absolute time' },
  'sui-mark':       { category: 'indicators', blurb: 'highlighted text run' },

  // ── Overlays ─────────────────────────────────────────────────────
  'sui-dialog':          { category: 'overlays', blurb: 'modal sheet' },
  'sui-alert-dialog':    { category: 'overlays', blurb: 'confirm / destructive prompt' },
  'sui-drawer':          { category: 'overlays', blurb: 'side-anchored panel' },
  'sui-popover':         { category: 'overlays', blurb: 'anchored floating content' },
  'sui-tooltip':         { category: 'overlays', blurb: 'hover label' },
  'sui-hover-card':      { category: 'overlays', blurb: 'rich hover preview' },
  'sui-dropdown-menu':   { category: 'overlays', blurb: 'click-triggered menu' },
  'sui-context-menu':    { category: 'overlays', blurb: 'right-click menu' },
  'sui-navigation-menu': { category: 'overlays', blurb: 'nested nav menu' },
  'sui-navigation-item': { category: 'overlays', blurb: 'nav menu entry' },
  'sui-accordion':       { category: 'overlays', blurb: 'vertical collapse group' },
  'sui-accordion-item':  { category: 'overlays', blurb: 'accordion section' },
  'sui-collapsible':     { category: 'overlays', blurb: 'single disclosure' },
  'sui-tabs':            { category: 'overlays', blurb: 'tab nav + panels' },
  'sui-tab':             { category: 'overlays', blurb: 'single tab trigger' },
  'sui-tab-panel':       { category: 'overlays', blurb: 'tab content region' },

  // ── Data ─────────────────────────────────────────────────────────
  'sui-table':          { category: 'data', blurb: 'tabular data' },
  'sui-list-tile':      { category: 'data', blurb: 'multi-line list row' },
  'sui-tree-view':      { category: 'data', blurb: 'nested hierarchy' },
  'sui-tree-item':      { category: 'data', blurb: 'tree node' },
  'sui-history-row':    { category: 'data', blurb: 'version-history entry' },
  'sui-breadcrumb':     { category: 'data', blurb: 'path trail' },
  'sui-breadcrumb-item':{ category: 'data', blurb: 'breadcrumb segment' },
  'sui-code':           { category: 'data', blurb: 'syntax-highlighted code' },
  'sui-diff':           { category: 'data', blurb: 'line-by-line diff' },
  'sui-json-view':      { category: 'data', blurb: 'collapsible JSON inspector' },
  'sui-markdown':       { category: 'data', blurb: 'rendered markdown' },
  'sui-timeline':       { category: 'data', blurb: 'event sequence' },
  'sui-timeline-item':  { category: 'data', blurb: 'timeline event' },

  // ── Charts ───────────────────────────────────────────────────────
  'sui-area-chart': { category: 'charts', blurb: 'filled trend chart' },
  'sui-bar-chart':  { category: 'charts', blurb: 'categorical bars' },
  'sui-line-chart': { category: 'charts', blurb: 'trend line chart' },
  'sui-donut':      { category: 'charts', blurb: 'proportional ring' },
  'sui-heatmap':    { category: 'charts', blurb: 'density grid' },
  'sui-sparkline':  { category: 'charts', blurb: 'inline micro-trend' },
  'sui-gauge':      { category: 'charts', blurb: 'radial value gauge' },
}

export function metaFor(tag: string): TagMeta {
  return TAG_META[tag] ?? { category: 'other', blurb: '' }
}
