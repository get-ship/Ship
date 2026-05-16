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
   * (sui-tab inside sui-tabs) or compositions that need surrounding
   * markup (sui-pipeline-dag + arrows + steps).
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
  'sui-button': [
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
  'sui-badge': [
    { label: 'Default',   attrs: {}, innerHTML: 'badge' },
    { label: 'Accent',    attrs: { tone: 'accent',      variant: 'soft' }, innerHTML: 'accent' },
    { label: 'Success',   attrs: { tone: 'success',     variant: 'soft' }, innerHTML: 'success' },
    { label: 'Warning',   attrs: { tone: 'warning',     variant: 'soft' }, innerHTML: 'warning' },
    { label: 'Destructive', attrs: { tone: 'destructive', variant: 'soft' }, innerHTML: 'destructive' },
    { label: 'Outline',   attrs: { variant: 'outline' }, innerHTML: 'outline' },
  ],
  'sui-input': [
    { label: 'Default',     attrs: { placeholder: 'you@example.com' } },
    { label: 'With value',  attrs: { value: 'acme-co' } },
    { label: 'Invalid',     attrs: { value: 'no-at-sign', invalid: '', 'error-message': 'Please enter a valid email.' } },
    { label: 'Disabled',    attrs: { value: 'read only', disabled: '' } },
  ],
  'sui-select': [
    { label: 'Default', innerHTML: '<option>Low</option><option selected>Medium</option><option>High</option>' },
  ],
  'sui-switch': [
    { label: 'Off' },
    { label: 'On', attrs: { checked: '' } },
  ],
  'sui-checkbox': [
    { label: 'Off' },
    { label: 'On', attrs: { checked: '' } },
    { label: 'Indeterminate', attrs: { indeterminate: '' } },
    { label: 'Disabled', attrs: { disabled: '' } },
  ],
  'sui-card': [
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
  'sui-alert': [
    { label: 'Info',        attrs: { tone: 'info' },        innerHTML: '<span slot="title">Heads up</span>Something informational is happening.' },
    { label: 'Success',     attrs: { tone: 'success' },     innerHTML: '<span slot="title">All good</span>Your changes were saved.' },
    { label: 'Warning',     attrs: { tone: 'warning' },     innerHTML: '<span slot="title">Careful</span>Double-check the config before proceeding.' },
    { label: 'Destructive', attrs: { tone: 'destructive', dismissible: '' }, innerHTML: '<span slot="title">Something went wrong</span>Check the logs and try again.' },
  ],
  'sui-status-dot': [
    { label: 'Default' },
    { label: 'Success', attrs: { tone: 'success', pulse: '' } },
    { label: 'Info',    attrs: { tone: 'info',    pulse: '' } },
    { label: 'Warning', attrs: { tone: 'warning' } },
    { label: 'Destructive', attrs: { tone: 'destructive' } },
  ],
  'sui-spinner': [
    { label: 'Default' },
    { label: 'Accent', attrs: { tone: 'accent' } },
  ],
  'sui-tabs': [
    {
      label: 'Underline',
      attrs: { active: 'one' },
      innerHTML: `
        <sui-tab name="one" label="Overview"></sui-tab>
        <sui-tab name="two" label="Activity" count="12"></sui-tab>
        <sui-tab name="three" label="Settings"></sui-tab>
        <sui-tab-panel name="one"><p style="padding:12px 0;color:var(--muted-foreground);">Overview content lives here.</p></sui-tab-panel>
        <sui-tab-panel name="two"><p style="padding:12px 0;color:var(--muted-foreground);">Activity feed.</p></sui-tab-panel>
        <sui-tab-panel name="three"><p style="padding:12px 0;color:var(--muted-foreground);">Settings pane.</p></sui-tab-panel>
      `,
    },
    {
      label: 'Segmented',
      attrs: { active: 'two', variant: 'segmented', size: 'sm' },
      innerHTML: `
        <sui-tab name="one" label="Day"></sui-tab>
        <sui-tab name="two" label="Week"></sui-tab>
        <sui-tab name="three" label="Month"></sui-tab>
      `,
    },
  ],
  'sui-empty-state': [
    {
      label: 'No items',
      innerHTML: `
        <span slot="title">Nothing here yet</span>
        When you create your first item it&rsquo;ll show up here.
      `,
    },
  ],
  'sui-stat': [
    {
      label: 'Default',
      innerHTML: `
        <span slot="label">Active runs</span>
        <span slot="value">128</span>
        <span slot="delta" style="color:oklch(0.6 0.16 150);">+12</span>
      `,
    },
  ],
  'sui-list-tile': [
    { label: 'Default', innerHTML: 'Pick a workspace' },
    { label: 'Selected', attrs: { selected: '' }, innerHTML: 'Selected row' },
    { label: 'Interactive', attrs: { interactive: '' }, innerHTML: 'Click me' },
    { label: 'Card', attrs: { variant: 'card' }, innerHTML: 'Card tile' },
  ],
  'sui-avatar': [
    { label: 'Initials', attrs: { name: 'Priya Nair' } },
    { label: 'Small',    attrs: { name: 'Sam',        size: 'sm', tone: 'accent' } },
  ],
  'sui-chip': [
    { label: 'Default',    attrs: {}, innerHTML: 'launch' },
    { label: 'Selected',   attrs: { selected: '', tone: 'accent' }, innerHTML: 'priority' },
    { label: 'Removable',  attrs: { removable: '', tone: 'info' }, innerHTML: 'mcp' },
  ],
  'sui-kbd': [
    { label: '⌘K',       attrs: { keys: '⌘ K' } },
    { label: 'Shift + /',  attrs: { keys: 'Shift ?' } },
  ],
  'sui-sparkline': [
    { label: 'Accent', attrs: { data: '3,7,5,11,8,14,10', tone: 'accent', width: '220', height: '36' } },
  ],
  'sui-gauge': [
    { label: '42%', attrs: { value: '42', min: '0', max: '100', tone: 'success', size: '120', label: '42%' } },
  ],
  'sui-progress-ring': [
    { label: '60%', attrs: { value: '60', size: '120', tone: 'primary' } },
  ],
  'sui-bar-chart': [
    { label: 'Weekly', attrs: { data: '3,7,5,11,8,14,10', labels: 'M,T,W,T,F,S,S', tone: 'primary', width: '260', height: '100' } },
  ],
  'sui-line-chart': [
    { label: 'Trend', attrs: { series: '[{"data":[3,7,5,11,8,14,10]}]', labels: 'M,T,W,T,F,S,S', width: '280', height: '100' } },
  ],
  'sui-pipeline-step': [
    { label: 'Completed', attrs: { name: 'collect',     'step-type': 'run',   state: 'completed', agent: 'git log' } },
    { label: 'Running',   attrs: { name: 'review',      'step-type': 'agent', state: 'running',   agent: 'Claude' } },
    { label: 'Pending',   attrs: { name: 'merge',       'step-type': 'merge', state: 'pending',   agent: '—' } },
    { label: 'Failed',    attrs: { name: 'tests',       'step-type': 'gate',  state: 'failed',    agent: '3 failures' } },
  ],

  /* Content-dependent primitives ------------------------------------ */
  'sui-label': [
    { label: 'Default',  attrs: { for: 'demo-input' }, innerHTML: 'Email' },
    { label: 'Required', attrs: { for: 'demo-input', required: '' }, innerHTML: 'Email' },
  ],
  'sui-field': [
    {
      label: 'With input',
      raw: `<sui-field>
  <sui-label slot="label">Email</sui-label>
  <sui-input placeholder="you@company.com"></sui-input>
</sui-field>`,
    },
    {
      label: 'Invalid',
      raw: `<sui-field invalid>
  <sui-label slot="label" required>Email</sui-label>
  <sui-input value="no-at-sign" invalid></sui-input>
  <span slot="error">Please enter a valid email.</span>
</sui-field>`,
    },
  ],
  'sui-progress': [
    { label: '40%',         raw: '<div style="width:220px;"><sui-progress value="40"  max="100" label="40%"></sui-progress></div>' },
    { label: 'Success 78%', raw: '<div style="width:220px;"><sui-progress value="78"  max="100" tone="success" label="78%"></sui-progress></div>' },
  ],
  'sui-separator': [
    { label: 'Horizontal', raw: '<div style="width:240px;"><sui-separator></sui-separator></div>' },
    { label: 'Dashed',     raw: '<div style="width:240px;"><sui-separator variant="dashed"></sui-separator></div>' },
    { label: 'Vertical',   raw: '<div style="display:inline-flex;align-items:center;gap:8px;height:24px;">Left<sui-separator orientation="vertical"></sui-separator>Right</div>' },
  ],
  'sui-breadcrumb': [
    {
      label: 'Default',
      raw: `<sui-breadcrumb>
  <sui-breadcrumb-item href="/">Home</sui-breadcrumb-item>
  <sui-breadcrumb-item href="/components">Components</sui-breadcrumb-item>
  <sui-breadcrumb-item active>sui-button</sui-breadcrumb-item>
</sui-breadcrumb>`,
    },
  ],
  'sui-breadcrumb-item': [
    { label: 'Link',   attrs: { href: '/components' }, innerHTML: 'Components' },
    { label: 'Active', attrs: { active: '' },          innerHTML: 'Active page' },
  ],
  'sui-icon': [
    { label: 'settings', attrs: { name: 'settings', size: 'md' } },
    { label: 'search',   attrs: { name: 'search',   size: 'md' } },
    { label: 'plus',     attrs: { name: 'plus',     size: 'md' } },
  ],
  'sui-link': [
    { label: 'Default',  attrs: { href: '#' }, innerHTML: 'Read the docs' },
    { label: 'External', attrs: { href: 'https://getship.dev', external: '' }, innerHTML: 'getship.dev' },
  ],
  'sui-skeleton': [
    { label: 'Text',   attrs: { lines: '3', width: '240px' } },
    { label: 'Avatar', attrs: { variant: 'circle', width: '40px', height: '40px' } },
    { label: 'Card',   attrs: { variant: 'rect', width: '240px', height: '60px' } },
  ],
  'sui-step-segments': [
    { label: 'Progress', attrs: { values: 'completed,completed,running,pending,pending', size: 'md' } },
  ],
  'sui-meter': [
    {
      label: 'Split',
      raw: `<div style="width:280px;"><sui-meter segments='[{"count":42,"tone":"success","label":"allowed"},{"count":28,"tone":"warning","label":"throttled"},{"count":12,"tone":"destructive","label":"denied"}]' total="100"></sui-meter></div>`,
    },
  ],
  'sui-heatmap': [
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
  'sui-diff': [
    { label: 'Inline', attrs: { from: 'hello world', to: 'hello there world' } },
  ],
  'sui-json-view': [
    {
      label: 'Collapsed',
      attrs: {
        data: '{"user":{"id":42,"name":"Priya","skills":["plan","review","qa"]},"active":true}',
        expanded: 'false',
      },
    },
  ],
  'sui-markdown': [
    {
      label: 'Rendered',
      attrs: {
        value: '# Release v1.2\n\n- worktree-per-agent dispatch\n- long-running cancel\n- breaking: `ship spawn` needs a skill',
      },
    },
  ],
  'sui-table': [
    {
      label: 'Compact',
      raw: `<sui-table variant="compact" hoverable>
  <thead><tr><th>Task</th><th>Status</th><th>Owner</th></tr></thead>
  <tbody>
    <tr><td>Ship release post</td><td><sui-badge tone="info" variant="soft" size="sm">review</sui-badge></td><td>priya</td></tr>
    <tr><td>Fix cancel flow</td><td><sui-badge tone="success" variant="soft" size="sm">merged</sui-badge></td><td>sam</td></tr>
    <tr><td>Worktree cleanup</td><td><sui-badge tone="warning" variant="soft" size="sm">draft</sui-badge></td><td>you</td></tr>
  </tbody>
</sui-table>`,
    },
  ],
  'sui-timestamp': [
    { label: 'ISO',     attrs: { value: new Date(Date.now() - 1000 * 60 * 17).toISOString() } },
    { label: 'Format',  attrs: { value: new Date(Date.now() - 1000 * 60 * 60 * 24 * 2).toISOString(), format: 'date' } },
  ],
  'sui-tree-view': [
    {
      label: 'File tree',
      raw: `<sui-tree-view>
  <sui-tree-node label=".ship" expanded>
    <sui-tree-node label="skills" expanded>
      <sui-tree-node label="review.md" leaf></sui-tree-node>
      <sui-tree-node label="plan.md" leaf></sui-tree-node>
    </sui-tree-node>
    <sui-tree-node label="views">
      <sui-tree-node label="dashboard" leaf></sui-tree-node>
    </sui-tree-node>
  </sui-tree-node>
</sui-tree-view>`,
    },
  ],
  'sui-timeline': [
    {
      label: 'Recent',
      raw: `<sui-timeline>
  <sui-timeline-item at="09:14" tone="success">Weekly digest sent</sui-timeline-item>
  <sui-timeline-item at="08:22" tone="info">Release post drafted</sui-timeline-item>
  <sui-timeline-item at="yesterday" tone="accent">PR #412 reviewed</sui-timeline-item>
</sui-timeline>`,
    },
  ],
  'sui-timeline-item': [
    { label: 'Default', attrs: { at: '09:14', tone: 'success' }, innerHTML: 'Weekly digest sent' },
  ],
  'sui-toolbar': [
    {
      label: 'Default',
      raw: `<sui-toolbar>
  <sui-button size="sm" variant="ghost">Cancel</sui-button>
  <span slot="spacer"></span>
  <sui-button size="sm" tone="primary" variant="solid">Save</sui-button>
</sui-toolbar>`,
    },
  ],

  /* Composite children — render them inside their parent ------------ */
  'sui-tab': [
    {
      label: 'Inside sui-tabs',
      raw: `<sui-tabs active="one">
  <sui-tab name="one" label="Overview"></sui-tab>
  <sui-tab name="two" label="Activity" count="12"></sui-tab>
  <sui-tab name="three" label="Settings"></sui-tab>
  <sui-tab-panel name="one"><p style="padding:8px 0;color:var(--muted-foreground);">Overview content.</p></sui-tab-panel>
</sui-tabs>`,
    },
  ],
  'sui-tab-panel': [
    {
      label: 'Inside sui-tabs',
      raw: `<sui-tabs active="two">
  <sui-tab name="one" label="One"></sui-tab>
  <sui-tab name="two" label="Two"></sui-tab>
  <sui-tab-panel name="one"><p style="padding:8px 0;color:var(--muted-foreground);">Panel one.</p></sui-tab-panel>
  <sui-tab-panel name="two"><p style="padding:8px 0;color:var(--foreground);">Panel two (active).</p></sui-tab-panel>
</sui-tabs>`,
    },
  ],

  /* Overlays — show a descriptive note instead of a broken preview -- */
  'sui-dialog': [
    { label: 'Modal', overlay: true, caption: 'Modal dialog. Opens via `open` attribute or `showModal()`.' },
  ],
  'sui-alert-dialog': [
    { label: 'Destructive', overlay: true, caption: 'Confirm-style alert dialog. Triggers via the `open` attribute.' },
  ],
  'sui-popover': [
    { label: 'Anchored', overlay: true, caption: 'Popover anchored to a trigger slot. Toggle via `open`.' },
  ],
  'sui-tooltip': [
    { label: 'On hover', overlay: true, caption: 'Tooltip that appears on hover / focus of its target.' },
  ],
  'sui-dropdown-menu': [
    { label: 'Menu', overlay: true, caption: 'Dropdown menu anchored to a trigger. Toggle via `open`.' },
  ],
  'sui-toast-container': [
    { label: 'Stack', overlay: true, caption: 'Host for toast notifications. Spawn toasts via `showToast()`.' },
  ],
  'sui-toast': [
    { label: 'Info',    attrs: { tone: 'info' },    innerHTML: 'Saved successfully' },
    { label: 'Warning', attrs: { tone: 'warning', dismissible: '' }, innerHTML: 'Approaching token limit' },
  ],

  /* Annotation elements — newly exposed; show inline notes --------- */
  'sui-annotation-layer': [
    { label: 'Wrapper', overlay: true, caption: 'Wraps a target region to capture annotations. Renders invisibly until activated.' },
  ],
  'sui-annotation-list': [
    { label: 'List', overlay: true, caption: 'Renders a list of staged annotations. Populated imperatively.' },
  ],
  'sui-annotation-marker': [
    { label: 'Marker', overlay: true, caption: 'Single annotation marker pin. Positioned by a parent layer.' },
  ],

  /* Domain compositions requiring parent/children ------------------ */
  'sui-pipeline-dag': [
    {
      label: 'Chain',
      raw: `<sui-pipeline-dag>
  <sui-pipeline-step name="collect" step-type="run" state="completed" agent="git log"></sui-pipeline-step>
  <span class="arrow">&rarr;</span>
  <sui-pipeline-step name="review" step-type="agent" state="completed" agent="Claude"></sui-pipeline-step>
  <span class="arrow">&rarr;</span>
  <sui-pipeline-step name="gate" step-type="gate" state="running" agent="acceptance"></sui-pipeline-step>
</sui-pipeline-dag>`,
    },
  ],
  'sui-git-diff': [
    {
      label: 'Diff',
      raw: `<sui-git-diff>
  <sui-git-diff-file path="src/index.ts" additions="4" deletions="1">
    <sui-git-diff-hunk header="@@ -10,3 +10,6 @@">
      <sui-git-diff-line type="context" old="10" new="10">const name = 'ship';</sui-git-diff-line>
      <sui-git-diff-line type="del" old="11">console.log('hi');</sui-git-diff-line>
      <sui-git-diff-line type="add" new="11">console.info('hello');</sui-git-diff-line>
      <sui-git-diff-line type="add" new="12">console.info('world');</sui-git-diff-line>
    </sui-git-diff-hunk>
  </sui-git-diff-file>
</sui-git-diff>`,
    },
  ],
  'sui-git-diff-file': [
    {
      label: 'File', raw: `<sui-git-diff-file path="src/button.ts" additions="2" deletions="1" collapsed>
  <sui-git-diff-hunk header="@@ -1,3 +1,4 @@"></sui-git-diff-hunk>
</sui-git-diff-file>`,
    },
  ],
  'sui-git-diff-hunk': [
    {
      label: 'Hunk', raw: `<sui-git-diff-hunk header="@@ -1,3 +1,4 @@">
  <sui-git-diff-line type="context" old="1" new="1">line unchanged</sui-git-diff-line>
  <sui-git-diff-line type="add"     new="2">line added</sui-git-diff-line>
</sui-git-diff-hunk>`,
    },
  ],
  'sui-git-diff-line': [
    {
      label: 'Inside hunk',
      raw: `<sui-git-diff-hunk header="@@ -1,2 +1,3 @@">
  <sui-git-diff-line type="add" new="2">console.log('added')</sui-git-diff-line>
</sui-git-diff-hunk>`,
    },
  ],
};

export function getShots(tag: string): ComponentShot[] {
  return SHOTS[tag] ?? [{ label: 'Default' }];
}
