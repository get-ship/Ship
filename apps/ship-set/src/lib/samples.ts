/**
 * Preview samples per tag. Each tag can have one or more "shots" —
 * a label + attrs + innerHTML that the per-component page renders
 * live. Tags without a sample fall back to a bare <tag/> preview so
 * the page still shows something, it just isn't opinionated.
 *
 * This is hand-authored rather than reused from the shell's old
 * gallery because shadcn-shaped component pages want tight, purposeful
 * variant strips, not a dump of every permutation. Where several
 * variants are useful, a tag gets multiple shots.
 */

export interface ComponentShot {
  label: string;
  attrs?: Record<string, string>;
  innerHTML?: string;
  /**
   * When set, the renderer emits this HTML directly instead of wrapping
   * in <tag …>. Use for components that only make sense inside a parent
   * (ship-tab inside ship-tabs) or compositions that need surrounding
   * markup (ship-pipeline-dag + arrows + steps).
   */
  raw?: string;
  /**
   * When true, the shot skips live rendering and instead shows a styled
   * "open this overlay to see it" note. Used for dialogs, popovers,
   * tooltips, and other top-layer components that don't render in a
   * grid preview.
   */
  overlay?: boolean;
  /** Caption shown with overlay-type shots. */
  caption?: string;
}

const btn = (label: string, tone = '', variant = '', size = '') => ({
  label,
  attrs: {
    ...(tone ? { tone } : {}),
    ...(variant ? { variant } : {}),
    ...(size ? { size } : {}),
  },
  innerHTML: label,
});

export const SHOTS: Record<string, ComponentShot[]> = {
  'ship-button': [
    btn('Primary',     'primary',     'solid'),
    btn('Accent',      'accent',      'solid'),
    btn('Secondary',   'default',     'outline'),
    btn('Ghost',       '',            'ghost'),
    btn('Destructive', 'destructive', 'soft'),
    btn('Small',       'primary',     'solid', 'sm'),
    btn('Large',       'primary',     'solid', 'lg'),
    { label: 'Loading', attrs: { tone: 'primary', variant: 'solid', loading: '' }, innerHTML: 'Saving' },
    { label: 'Disabled', attrs: { tone: 'primary', variant: 'solid', disabled: '' }, innerHTML: 'Disabled' },
  ],
  'ship-badge': [
    { label: 'Default',   attrs: {}, innerHTML: 'badge' },
    { label: 'Accent',    attrs: { tone: 'accent',      variant: 'soft' }, innerHTML: 'accent' },
    { label: 'Success',   attrs: { tone: 'success',     variant: 'soft' }, innerHTML: 'success' },
    { label: 'Warning',   attrs: { tone: 'warning',     variant: 'soft' }, innerHTML: 'warning' },
    { label: 'Destructive', attrs: { tone: 'destructive', variant: 'soft' }, innerHTML: 'destructive' },
    { label: 'Outline',   attrs: { variant: 'outline' }, innerHTML: 'outline' },
  ],
  'ship-input': [
    { label: 'Default',     attrs: { placeholder: 'you@example.com' } },
    { label: 'With value',  attrs: { value: 'acme-co' } },
    { label: 'Invalid',     attrs: { value: 'no-at-sign', invalid: '', 'error-message': 'Please enter a valid email.' } },
    { label: 'Disabled',    attrs: { value: 'read only', disabled: '' } },
  ],
  'ship-select': [
    { label: 'Default', innerHTML: '<option>Low</option><option selected>Medium</option><option>High</option>' },
  ],
  'ship-switch': [
    { label: 'Off' },
    { label: 'On', attrs: { checked: '' } },
  ],
  'ship-checkbox': [
    { label: 'Off' },
    { label: 'On', attrs: { checked: '' } },
    { label: 'Indeterminate', attrs: { indeterminate: '' } },
    { label: 'Disabled', attrs: { disabled: '' } },
  ],
  'ship-card': [
    {
      label: 'Default',
      attrs: { padding: 'lg' },
      innerHTML: '<strong style="font-family:var(--font-display);font-size:16px;">Card title</strong><p style="color:var(--muted-foreground);margin-top:6px;font-size:13px;line-height:1.5;">A container surface with a border, radius, and shadow. Use it anywhere.</p>',
    },
    {
      label: 'Outline',
      attrs: { variant: 'outline', padding: 'lg' },
      innerHTML: '<strong style="font-family:var(--font-display);font-size:16px;">Outline card</strong><p style="color:var(--muted-foreground);margin-top:6px;font-size:13px;line-height:1.5;">Muted border, no shadow — sits inside another container.</p>',
    },
    {
      label: 'Status rail',
      attrs: { variant: 'outline', 'status-rail': '', tone: 'success', padding: 'lg' },
      innerHTML: '<strong style="font-family:var(--font-display);font-size:16px;">Status rail</strong><p style="color:var(--muted-foreground);margin-top:6px;font-size:13px;line-height:1.5;">3px tone-tinted rail on the left edge.</p>',
    },
  ],
  'ship-alert': [
    { label: 'Info',        attrs: { tone: 'info' },        innerHTML: '<span slot="title">Heads up</span>Something informational is happening.' },
    { label: 'Success',     attrs: { tone: 'success' },     innerHTML: '<span slot="title">All good</span>Your changes were saved.' },
    { label: 'Warning',     attrs: { tone: 'warning' },     innerHTML: '<span slot="title">Careful</span>Double-check the config before proceeding.' },
    { label: 'Destructive', attrs: { tone: 'destructive', dismissible: '' }, innerHTML: '<span slot="title">Something went wrong</span>Check the logs and try again.' },
  ],
  'ship-status-dot': [
    { label: 'Default' },
    { label: 'Success', attrs: { tone: 'success', pulse: '' } },
    { label: 'Info',    attrs: { tone: 'info',    pulse: '' } },
    { label: 'Warning', attrs: { tone: 'warning' } },
    { label: 'Destructive', attrs: { tone: 'destructive' } },
  ],
  'ship-spinner': [
    { label: 'Default' },
    { label: 'Accent', attrs: { tone: 'accent' } },
  ],
  'ship-progress': [
    { label: '40%', attrs: { value: '40', max: '100' } },
    { label: '78% success', attrs: { value: '78', max: '100', tone: 'success' } },
  ],
  'ship-separator': [
    { label: 'Horizontal', attrs: {} },
    { label: 'Dashed',     attrs: { variant: 'dashed' } },
  ],
  'ship-tabs': [
    {
      label: 'Underline',
      attrs: { active: 'one' },
      innerHTML: `
        <ship-tab name="one" label="Overview"></ship-tab>
        <ship-tab name="two" label="Activity" count="12"></ship-tab>
        <ship-tab name="three" label="Settings"></ship-tab>
        <ship-tab-panel name="one"><p style="padding:12px 0;color:var(--muted-foreground);">Overview content lives here.</p></ship-tab-panel>
        <ship-tab-panel name="two"><p style="padding:12px 0;color:var(--muted-foreground);">Activity feed.</p></ship-tab-panel>
        <ship-tab-panel name="three"><p style="padding:12px 0;color:var(--muted-foreground);">Settings pane.</p></ship-tab-panel>
      `,
    },
    {
      label: 'Segmented',
      attrs: { active: 'two', variant: 'segmented', size: 'sm' },
      innerHTML: `
        <ship-tab name="one" label="Day"></ship-tab>
        <ship-tab name="two" label="Week"></ship-tab>
        <ship-tab name="three" label="Month"></ship-tab>
      `,
    },
  ],
  'ship-toolbar': [
    {
      label: 'Default',
      innerHTML: `
        <ship-button size="sm" variant="ghost">Cancel</ship-button>
        <span slot="spacer"></span>
        <ship-button size="sm" tone="primary" variant="solid">Save</ship-button>
      `,
    },
  ],
  'ship-empty-state': [
    {
      label: 'No items',
      innerHTML: `
        <span slot="title">Nothing here yet</span>
        When you create your first item it&rsquo;ll show up here.
      `,
    },
  ],
  'ship-stat': [
    {
      label: 'Default',
      innerHTML: `
        <span slot="label">Active runs</span>
        <span slot="value">128</span>
        <span slot="delta" style="color:oklch(0.6 0.16 150);">+12</span>
      `,
    },
  ],
  'ship-list-tile': [
    { label: 'Default', innerHTML: 'Pick a workspace' },
    { label: 'Selected', attrs: { selected: '' }, innerHTML: 'Selected row' },
    { label: 'Interactive', attrs: { interactive: '' }, innerHTML: 'Click me' },
    { label: 'Card', attrs: { variant: 'card' }, innerHTML: 'Card tile' },
  ],
  'ship-avatar': [
    { label: 'Initials', attrs: { name: 'Priya Nair' } },
    { label: 'Small',    attrs: { name: 'Sam',        size: 'sm', tone: 'accent' } },
  ],
  'ship-chip': [
    { label: 'Default',    attrs: {}, innerHTML: 'launch' },
    { label: 'Selected',   attrs: { selected: '', tone: 'accent' }, innerHTML: 'priority' },
    { label: 'Removable',  attrs: { removable: '', tone: 'info' }, innerHTML: 'mcp' },
  ],
  'ship-kbd': [
    { label: '⌘K',       attrs: { keys: '⌘ K' } },
    { label: 'Shift + /',  attrs: { keys: 'Shift ?' } },
  ],
  'ship-sparkline': [
    { label: 'Accent', attrs: { data: '3,7,5,11,8,14,10', tone: 'accent', width: '220', height: '36' } },
  ],
  'ship-gauge': [
    { label: '42%', attrs: { value: '42', min: '0', max: '100', tone: 'success', size: '120', label: '42%' } },
  ],
  'ship-progress-ring': [
    { label: '60%', attrs: { value: '60', size: '120', tone: 'primary' } },
  ],
  'ship-bar-chart': [
    { label: 'Weekly', attrs: { data: '3,7,5,11,8,14,10', labels: 'M,T,W,T,F,S,S', tone: 'primary', width: '260', height: '100' } },
  ],
  'ship-line-chart': [
    { label: 'Trend', attrs: { series: '[{"data":[3,7,5,11,8,14,10]}]', labels: 'M,T,W,T,F,S,S', width: '280', height: '100' } },
  ],
  'ship-timeline': [
    {
      label: 'Recent',
      innerHTML: `
        <ship-timeline-item at="09:14" tone="success">Weekly digest sent</ship-timeline-item>
        <ship-timeline-item at="08:22" tone="info">Release post drafted</ship-timeline-item>
        <ship-timeline-item at="yesterday" tone="accent">PR #412 reviewed</ship-timeline-item>
      `,
    },
  ],
  'ship-pipeline-step': [
    { label: 'Completed', attrs: { name: 'collect',     'step-type': 'run',   state: 'completed', agent: 'git log' } },
    { label: 'Running',   attrs: { name: 'review',      'step-type': 'agent', state: 'running',   agent: 'Claude' } },
    { label: 'Pending',   attrs: { name: 'merge',       'step-type': 'merge', state: 'pending',   agent: '—' } },
    { label: 'Failed',    attrs: { name: 'tests',       'step-type': 'gate',  state: 'failed',    agent: '3 failures' } },
  ],

  /* Content-dependent primitives ------------------------------------ */
  'ship-label': [
    { label: 'Default',  attrs: { for: 'demo-input' }, innerHTML: 'Email' },
    { label: 'Required', attrs: { for: 'demo-input', required: '' }, innerHTML: 'Email' },
  ],
  'ship-field': [
    {
      label: 'With input',
      raw: `<ship-field>
  <ship-label slot="label">Email</ship-label>
  <ship-input placeholder="you@company.com"></ship-input>
</ship-field>`,
    },
    {
      label: 'Invalid',
      raw: `<ship-field invalid>
  <ship-label slot="label" required>Email</ship-label>
  <ship-input value="no-at-sign" invalid></ship-input>
  <span slot="error">Please enter a valid email.</span>
</ship-field>`,
    },
  ],
  'ship-progress': [
    { label: '40%',         raw: '<div style="width:220px;"><ship-progress value="40"  max="100" label="40%"></ship-progress></div>' },
    { label: 'Success 78%', raw: '<div style="width:220px;"><ship-progress value="78"  max="100" tone="success" label="78%"></ship-progress></div>' },
  ],
  'ship-separator': [
    { label: 'Horizontal', raw: '<div style="width:240px;"><ship-separator></ship-separator></div>' },
    { label: 'Dashed',     raw: '<div style="width:240px;"><ship-separator variant="dashed"></ship-separator></div>' },
    { label: 'Vertical',   raw: '<div style="display:inline-flex;align-items:center;gap:8px;height:24px;">Left<ship-separator orientation="vertical"></ship-separator>Right</div>' },
  ],
  'ship-breadcrumb': [
    {
      label: 'Default',
      raw: `<ship-breadcrumb>
  <ship-breadcrumb-item href="/">Home</ship-breadcrumb-item>
  <ship-breadcrumb-item href="/components">Components</ship-breadcrumb-item>
  <ship-breadcrumb-item active>ship-button</ship-breadcrumb-item>
</ship-breadcrumb>`,
    },
  ],
  'ship-breadcrumb-item': [
    { label: 'Link',   attrs: { href: '/components' }, innerHTML: 'Components' },
    { label: 'Active', attrs: { active: '' },          innerHTML: 'Active page' },
  ],
  'ship-icon': [
    { label: 'settings', attrs: { name: 'settings', size: 'md' } },
    { label: 'search',   attrs: { name: 'search',   size: 'md' } },
    { label: 'plus',     attrs: { name: 'plus',     size: 'md' } },
  ],
  'ship-link': [
    { label: 'Default',  attrs: { href: '#' }, innerHTML: 'Read the docs' },
    { label: 'External', attrs: { href: 'https://getship.dev', external: '' }, innerHTML: 'getship.dev' },
  ],
  'ship-skeleton': [
    { label: 'Text',   attrs: { lines: '3', width: '240px' } },
    { label: 'Avatar', attrs: { variant: 'circle', width: '40px', height: '40px' } },
    { label: 'Card',   attrs: { variant: 'rect', width: '240px', height: '60px' } },
  ],
  'ship-step-segments': [
    { label: 'Progress', attrs: { values: 'completed,completed,running,pending,pending', size: 'md' } },
  ],
  'ship-meter': [
    {
      label: 'Split',
      raw: `<div style="width:280px;"><ship-meter segments='[{"count":42,"tone":"success","label":"allowed"},{"count":28,"tone":"warning","label":"throttled"},{"count":12,"tone":"destructive","label":"denied"}]' total="100"></ship-meter></div>`,
    },
  ],
  'ship-heatmap': [
    {
      label: '7x12 grid',
      attrs: {
        data: '[[0,1,2,1,0,0,1],[1,2,3,2,1,0,1],[2,3,4,3,2,1,2],[3,4,5,4,3,2,3],[2,3,4,3,2,1,2]]',
        rows: '5',
        cols: '7',
        tone: 'primary',
      },
    },
  ],
  'ship-diff': [
    { label: 'Inline', attrs: { from: 'hello world', to: 'hello there world' } },
  ],
  'ship-json-view': [
    {
      label: 'Collapsed',
      attrs: {
        data: '{"user":{"id":42,"name":"Priya","skills":["plan","review","qa"]},"active":true}',
        expanded: 'false',
      },
    },
  ],
  'ship-markdown': [
    {
      label: 'Rendered',
      attrs: {
        value: '# Release v1.2\n\n- worktree-per-agent dispatch\n- long-running cancel\n- breaking: `ship spawn` needs a skill',
      },
    },
  ],
  'ship-table': [
    {
      label: 'Compact',
      raw: `<ship-table variant="compact" hoverable>
  <thead><tr><th>Task</th><th>Status</th><th>Owner</th></tr></thead>
  <tbody>
    <tr><td>Ship release post</td><td><ship-badge tone="info" variant="soft" size="sm">review</ship-badge></td><td>priya</td></tr>
    <tr><td>Fix cancel flow</td><td><ship-badge tone="success" variant="soft" size="sm">merged</ship-badge></td><td>sam</td></tr>
    <tr><td>Worktree cleanup</td><td><ship-badge tone="warning" variant="soft" size="sm">draft</ship-badge></td><td>you</td></tr>
  </tbody>
</ship-table>`,
    },
  ],
  'ship-timestamp': [
    { label: 'ISO',     attrs: { value: new Date(Date.now() - 1000 * 60 * 17).toISOString() } },
    { label: 'Format',  attrs: { value: new Date(Date.now() - 1000 * 60 * 60 * 24 * 2).toISOString(), format: 'date' } },
  ],
  'ship-tree-view': [
    {
      label: 'File tree',
      raw: `<ship-tree-view>
  <ship-tree-node label=".ship" expanded>
    <ship-tree-node label="skills" expanded>
      <ship-tree-node label="review.md" leaf></ship-tree-node>
      <ship-tree-node label="plan.md" leaf></ship-tree-node>
    </ship-tree-node>
    <ship-tree-node label="views">
      <ship-tree-node label="dashboard" leaf></ship-tree-node>
    </ship-tree-node>
  </ship-tree-node>
</ship-tree-view>`,
    },
  ],
  'ship-timeline': [
    {
      label: 'Recent',
      raw: `<ship-timeline>
  <ship-timeline-item at="09:14" tone="success">Weekly digest sent</ship-timeline-item>
  <ship-timeline-item at="08:22" tone="info">Release post drafted</ship-timeline-item>
  <ship-timeline-item at="yesterday" tone="accent">PR #412 reviewed</ship-timeline-item>
</ship-timeline>`,
    },
  ],
  'ship-timeline-item': [
    { label: 'Default', attrs: { at: '09:14', tone: 'success' }, innerHTML: 'Weekly digest sent' },
  ],
  'ship-toolbar': [
    {
      label: 'Default',
      raw: `<ship-toolbar>
  <ship-button size="sm" variant="ghost">Cancel</ship-button>
  <span slot="spacer"></span>
  <ship-button size="sm" tone="primary" variant="solid">Save</ship-button>
</ship-toolbar>`,
    },
  ],

  /* Composite children — render them inside their parent ------------ */
  'ship-tab': [
    {
      label: 'Inside ship-tabs',
      raw: `<ship-tabs active="one">
  <ship-tab name="one" label="Overview"></ship-tab>
  <ship-tab name="two" label="Activity" count="12"></ship-tab>
  <ship-tab name="three" label="Settings"></ship-tab>
  <ship-tab-panel name="one"><p style="padding:8px 0;color:var(--muted-foreground);">Overview content.</p></ship-tab-panel>
</ship-tabs>`,
    },
  ],
  'ship-tab-panel': [
    {
      label: 'Inside ship-tabs',
      raw: `<ship-tabs active="two">
  <ship-tab name="one" label="One"></ship-tab>
  <ship-tab name="two" label="Two"></ship-tab>
  <ship-tab-panel name="one"><p style="padding:8px 0;color:var(--muted-foreground);">Panel one.</p></ship-tab-panel>
  <ship-tab-panel name="two"><p style="padding:8px 0;color:var(--foreground);">Panel two (active).</p></ship-tab-panel>
</ship-tabs>`,
    },
  ],

  /* Overlays — show a descriptive note instead of a broken preview -- */
  'ship-dialog': [
    { label: 'Modal', overlay: true, caption: 'Modal dialog. Opens via `open` attribute or `showModal()`.' },
  ],
  'ship-alert-dialog': [
    { label: 'Destructive', overlay: true, caption: 'Confirm-style alert dialog. Triggers via the `open` attribute.' },
  ],
  'ship-popover': [
    { label: 'Anchored', overlay: true, caption: 'Popover anchored to a trigger slot. Toggle via `open`.' },
  ],
  'ship-tooltip': [
    { label: 'On hover', overlay: true, caption: 'Tooltip that appears on hover / focus of its target.' },
  ],
  'ship-dropdown-menu': [
    { label: 'Menu', overlay: true, caption: 'Dropdown menu anchored to a trigger. Toggle via `open`.' },
  ],
  'ship-toast-container': [
    { label: 'Stack', overlay: true, caption: 'Host for toast notifications. Spawn toasts via `showToast()`.' },
  ],
  'ship-toast': [
    { label: 'Info',    attrs: { tone: 'info' },    innerHTML: 'Saved successfully' },
    { label: 'Warning', attrs: { tone: 'warning', dismissible: '' }, innerHTML: 'Approaching token limit' },
  ],

  /* Annotation elements — newly exposed; show inline notes --------- */
  'ship-annotation-layer': [
    { label: 'Wrapper', overlay: true, caption: 'Wraps a target region to capture annotations. Renders invisibly until activated.' },
  ],
  'ship-annotation-list': [
    { label: 'List', overlay: true, caption: 'Renders a list of staged annotations. Populated imperatively.' },
  ],
  'ship-annotation-marker': [
    { label: 'Marker', overlay: true, caption: 'Single annotation marker pin. Positioned by a parent layer.' },
  ],

  /* Domain compositions requiring parent/children ------------------ */
  'ship-pipeline-dag': [
    {
      label: 'Chain',
      raw: `<ship-pipeline-dag>
  <ship-pipeline-step name="collect" step-type="run" state="completed" agent="git log"></ship-pipeline-step>
  <span class="arrow">&rarr;</span>
  <ship-pipeline-step name="review" step-type="agent" state="completed" agent="Claude"></ship-pipeline-step>
  <span class="arrow">&rarr;</span>
  <ship-pipeline-step name="gate" step-type="gate" state="running" agent="acceptance"></ship-pipeline-step>
</ship-pipeline-dag>`,
    },
  ],
  'ship-git-diff': [
    {
      label: 'Diff',
      raw: `<ship-git-diff>
  <ship-git-diff-file path="src/index.ts" additions="4" deletions="1">
    <ship-git-diff-hunk header="@@ -10,3 +10,6 @@">
      <ship-git-diff-line type="context" old="10" new="10">const name = 'ship';</ship-git-diff-line>
      <ship-git-diff-line type="del" old="11">console.log('hi');</ship-git-diff-line>
      <ship-git-diff-line type="add" new="11">console.info('hello');</ship-git-diff-line>
      <ship-git-diff-line type="add" new="12">console.info('world');</ship-git-diff-line>
    </ship-git-diff-hunk>
  </ship-git-diff-file>
</ship-git-diff>`,
    },
  ],
  'ship-git-diff-file': [
    {
      label: 'File', raw: `<ship-git-diff-file path="src/button.ts" additions="2" deletions="1" collapsed>
  <ship-git-diff-hunk header="@@ -1,3 +1,4 @@"></ship-git-diff-hunk>
</ship-git-diff-file>`,
    },
  ],
  'ship-git-diff-hunk': [
    {
      label: 'Hunk', raw: `<ship-git-diff-hunk header="@@ -1,3 +1,4 @@">
  <ship-git-diff-line type="context" old="1" new="1">line unchanged</ship-git-diff-line>
  <ship-git-diff-line type="add"     new="2">line added</ship-git-diff-line>
</ship-git-diff-hunk>`,
    },
  ],
  'ship-git-diff-line': [
    {
      label: 'Inside hunk',
      raw: `<ship-git-diff-hunk header="@@ -1,2 +1,3 @@">
  <ship-git-diff-line type="add" new="2">console.log('added')</ship-git-diff-line>
</ship-git-diff-hunk>`,
    },
  ],
};

export function getShots(tag: string): ComponentShot[] {
  return SHOTS[tag] ?? [{ label: 'Default' }];
}
