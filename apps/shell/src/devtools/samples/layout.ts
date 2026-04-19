/**
 * Layout primitive samples: card / separator / toolbar / breadcrumb /
 * tabs / accordion / collapsible / scroll-area / navigation-menu /
 * tree-view / timeline / list-item.
 */

import { row, type Sample } from './types'

export const LAYOUT: Record<string, Sample[]> = {
  'ship-card': [
    {
      raw: true,
      innerHTML:
        '<div class="sample-triptych">' +
        '<ship-card tone="info" accent-bar><strong>Info card</strong><p style="margin:6px 0 0;color:var(--muted-foreground)">Accent bar fades in on hover.</p></ship-card>' +
        '<ship-card tone="success" status-rail fill><strong>Status rail + fill</strong><p style="margin:6px 0 0;color:var(--muted-foreground)">Tone-tinted wash.</p></ship-card>' +
        '<ship-card variant="outline"><strong>Outline</strong><p style="margin:6px 0 0;color:var(--muted-foreground)">Quieter nested card.</p></ship-card>' +
        '</div>',
      metaAttrs: { accent: 'info|success|outline' },
    },
  ],
  'ship-separator': [
    {
      innerHTML:
        '<div style="width:200px">' +
        'above<ship-separator></ship-separator>below' +
        '<ship-separator variant="dashed"></ship-separator>dashed' +
        '</div>',
    },
  ],
  'ship-toolbar': [
    {
      props: { variant: 'bordered', justify: 'between' },
      innerHTML:
        '<ship-button variant="ghost" size="sm">Save</ship-button>' +
        '<ship-button variant="ghost" size="sm">Undo</ship-button>' +
        '<ship-separator orientation="vertical"></ship-separator>' +
        '<span slot="spacer"></span>' +
        '<ship-button tone="info" size="sm">Run</ship-button>',
    },
  ],
  'ship-breadcrumb': [
    {
      innerHTML:
        '<ship-breadcrumb-item href="#">Ship</ship-breadcrumb-item>' +
        '<ship-breadcrumb-item href="#">Pipelines</ship-breadcrumb-item>' +
        '<ship-breadcrumb-item href="#">fix-auth</ship-breadcrumb-item>' +
        '<ship-breadcrumb-item active>Run 142</ship-breadcrumb-item>',
    },
  ],
  'ship-tabs': [
    {
      props: { active: 'overview' },
      innerHTML:
        '<ship-tab name="overview" label="Overview"></ship-tab>' +
        '<ship-tab name="runs" label="Runs" count="14"></ship-tab>' +
        '<ship-tab name="history" label="History"></ship-tab>' +
        '<ship-tab-panel name="overview">Overview panel content.</ship-tab-panel>' +
        '<ship-tab-panel name="runs">Runs panel content.</ship-tab-panel>' +
        '<ship-tab-panel name="history">History panel content.</ship-tab-panel>',
      metaAttrs: { variant: 'underline', active: 'overview' },
    },
    {
      props: { variant: 'segmented', active: 'day', tone: 'accent' },
      innerHTML:
        '<ship-tab name="day" label="Day"></ship-tab>' +
        '<ship-tab name="week" label="Week"></ship-tab>' +
        '<ship-tab name="month" label="Month"></ship-tab>' +
        '<ship-tab-panel name="day">Day.</ship-tab-panel>' +
        '<ship-tab-panel name="week">Week.</ship-tab-panel>' +
        '<ship-tab-panel name="month">Month.</ship-tab-panel>',
      metaAttrs: { variant: 'segmented', tone: 'accent' },
    },
  ],
  'ship-tree-view': [
    {
      innerHTML:
        '<ship-tree-node label="apps" expanded>' +
        '<ship-tree-node label="shell" expanded>' +
        '<ship-tree-node label="devtools.html" leaf></ship-tree-node>' +
        '<ship-tree-node label="src" expanded>' +
        '<ship-tree-node label="main.ts" leaf></ship-tree-node>' +
        '<ship-tree-node label="gallery.ts" leaf></ship-tree-node>' +
        '</ship-tree-node>' +
        '</ship-tree-node>' +
        '<ship-tree-node label="web" leaf></ship-tree-node>' +
        '</ship-tree-node>' +
        '<ship-tree-node label="packages" expanded>' +
        '<ship-tree-node label="ui" leaf></ship-tree-node>' +
        '</ship-tree-node>',
    },
  ],
  'ship-timeline': [
    {
      innerHTML:
        '<ship-timeline-item tone="info" at="10:00"><span slot="title">Pipeline dispatched</span>rust-lane picked up fix-auth</ship-timeline-item>' +
        '<ship-timeline-item tone="success" at="10:05"><span slot="title">Tests passed</span>142/142 — 14.2s</ship-timeline-item>' +
        '<ship-timeline-item tone="warning" at="10:07"><span slot="title">Gate review</span>Awaiting human approval…</ship-timeline-item>' +
        '<ship-timeline-item tone="accent" at="10:09"><span slot="title">Merged</span>to main · 2m duration</ship-timeline-item>',
    },
  ],
  'ship-list-item': [
    row(
      [
        '<ship-list-item style="width:240px"><span slot="leading">◆</span>A standard row<span slot="meta">secondary line</span></ship-list-item>',
        '<ship-list-item selected style="width:240px">Selected row<span slot="trailing">42</span></ship-list-item>',
        '<ship-list-item disabled style="width:240px">Disabled row</ship-list-item>',
      ].join(''),
    ),
  ],
  'ship-accordion': [
    {
      props: { type: 'single', value: 'first' },
      innerHTML:
        '<ship-accordion-item value="first">' +
        '<span slot="trigger">What is Ship?</span>' +
        'Ship is an agent-native CI/CD runtime that orchestrates pipelines.' +
        '</ship-accordion-item>' +
        '<ship-accordion-item value="second">' +
        '<span slot="trigger">How does the daemon work?</span>' +
        'The daemon runs locally and communicates over gRPC with the CLI and MCP server.' +
        '</ship-accordion-item>' +
        '<ship-accordion-item value="third">' +
        '<span slot="trigger">Can I extend it?</span>' +
        'Yes — write custom pipeline steps as shell commands or Rust functions.' +
        '</ship-accordion-item>',
      metaAttrs: { type: 'single', value: 'first' },
    },
  ],
  'ship-collapsible': [
    {
      props: { open: '' },
      innerHTML:
        '<ship-button slot="trigger" variant="outline" size="sm">Toggle details</ship-button>' +
        '<div style="padding:10px 0;color:var(--muted-foreground);font-size:var(--text-sm)">' +
        'This content is collapsible. Click the button above to show or hide it. ' +
        'Smooth height animation transitions the region open and closed.' +
        '</div>',
      metaAttrs: { open: 'true' },
    },
  ],
  'ship-scroll-area': [
    {
      props: { style: 'height:140px;width:260px' },
      innerHTML: Array.from({ length: 20 }, (_, i) =>
        `<div style="padding:4px 8px;font-size:var(--text-sm);color:var(--muted-foreground);border-bottom:1px solid var(--border)">Line ${i + 1} — scrollable content</div>`,
      ).join(''),
      metaAttrs: { orientation: 'vertical' },
    },
  ],
  'ship-navigation-menu': [
    {
      props: { value: 'overview' },
      innerHTML:
        '<ship-navigation-item value="overview">Overview</ship-navigation-item>' +
        '<ship-navigation-item value="pipelines">Pipelines</ship-navigation-item>' +
        '<ship-navigation-item value="agents">Agents</ship-navigation-item>' +
        '<ship-navigation-item value="settings" disabled>Settings</ship-navigation-item>',
      metaAttrs: { value: 'overview' },
    },
  ],
}
