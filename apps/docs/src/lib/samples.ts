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
    { label: 'On',  attrs: { checked: '' } },
    { label: 'Off' },
    { label: 'Disabled', attrs: { disabled: '' } },
  ],
  'sui-checkbox': [
    { label: 'On', attrs: { checked: '' } },
    { label: 'Off' },
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
    { label: 'Success', attrs: { tone: 'success', pulse: '' } },
    { label: 'Info',    attrs: { tone: 'info',    pulse: '' } },
    { label: 'Warning', attrs: { tone: 'warning' } },
    { label: 'Destructive', attrs: { tone: 'destructive' } },
    { label: 'Idle',    attrs: {} },
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
    { label: 'Accent', raw: '<div style="width:220px;"><sui-sparkline data="3,7,5,11,8,14,10" tone="accent" height="36"></sui-sparkline></div>' },
    { label: 'Filled', raw: '<div style="width:220px;"><sui-sparkline data="3,7,5,11,8,14,10" tone="primary" variant="area" height="36"></sui-sparkline></div>' },
  ],
  'sui-gauge': [
    { label: '42%', attrs: { value: '42', min: '0', max: '100', tone: 'success', size: '120', label: '42%' } },
    { label: '88%', attrs: { value: '88', min: '0', max: '100', tone: 'warning', size: '120', label: '88%' } },
  ],
  'sui-progress-ring': [
    { label: '60%', attrs: { value: '60', size: '120', tone: 'primary' } },
    { label: '92%', attrs: { value: '92', size: '120', tone: 'success' } },
  ],
  'sui-donut': [
    {
      label: 'Default',
      attrs: { value: '64', min: '0', max: '100', tone: 'accent', size: '120', label: '64%' },
    },
  ],
  'sui-bar-chart': [
    { label: 'Weekly', raw: '<div style="width:280px;"><sui-bar-chart data="3,7,5,11,8,14,10" labels="M,T,W,T,F,S,S" tone="primary" height="100"></sui-bar-chart></div>' },
  ],
  'sui-line-chart': [
    { label: 'Trend', raw: `<div style="width:300px;"><sui-line-chart series='[{"data":[3,7,5,11,8,14,10]}]' labels="M,T,W,T,F,S,S" height="100"></sui-line-chart></div>` },
  ],
  'sui-area-chart': [
    { label: 'Filled', raw: `<div style="width:300px;"><sui-area-chart series='[{"data":[3,7,5,11,8,14,10]}]' labels="M,T,W,T,F,S,S" tone="primary" height="120"></sui-area-chart></div>` },
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
    { label: 'Text',   raw: '<div style="width:240px;"><sui-skeleton shape="text" size="md"></sui-skeleton></div>' },
    { label: 'Line',   raw: '<div style="width:240px;"><sui-skeleton shape="line" size="lg"></sui-skeleton></div>' },
    { label: 'Avatar', attrs: { shape: 'circle', size: 'lg' } },
    { label: 'Block',  raw: '<div style="width:240px;"><sui-skeleton shape="block" size="md"></sui-skeleton></div>' },
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
      label: '5x7 grid',
      raw: `<sui-heatmap style="display:inline-block;width:158px;height:112px;" data="[[0,1,2,1,0,0,1],[1,2,3,2,1,0,1],[2,3,4,3,2,1,2],[3,4,5,4,3,2,3],[2,3,4,3,2,1,2]]" rows="5" cols="7" tone="primary"></sui-heatmap>`,
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
      raw: `<div style="width:260px;text-align:left;"><sui-tree-view>
  <sui-tree-item label=".ship" expanded>
    <sui-tree-item label="skills" expanded>
      <sui-tree-item label="review.md" leaf></sui-tree-item>
      <sui-tree-item label="plan.md" leaf></sui-tree-item>
    </sui-tree-item>
    <sui-tree-item label="views">
      <sui-tree-item label="dashboard" leaf></sui-tree-item>
    </sui-tree-item>
  </sui-tree-item>
</sui-tree-view></div>`,
    },
  ],
  'sui-tree-item': [
    {
      label: 'Inside sui-tree-view',
      raw: `<div style="width:240px;text-align:left;"><sui-tree-view>
  <sui-tree-item label="docs" expanded>
    <sui-tree-item label="getting-started.md" leaf></sui-tree-item>
    <sui-tree-item label="architecture.md" leaf></sui-tree-item>
  </sui-tree-item>
</sui-tree-view></div>`,
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
  'sui-toggle': [
    { label: 'On', attrs: { pressed: '' }, innerHTML: 'Pressed' },
    { label: 'Off', innerHTML: 'Idle' },
    { label: 'Disabled', attrs: { disabled: '' }, innerHTML: 'Disabled' },
  ],
  'sui-toggle-group': [
    {
      label: 'View mode',
      raw: `<sui-toggle-group>
  <sui-toggle value="list" pressed>List</sui-toggle>
  <sui-toggle value="grid">Grid</sui-toggle>
  <sui-toggle value="kanban">Kanban</sui-toggle>
</sui-toggle-group>`,
    },
  ],
  'sui-radio': [
    { label: 'On',  attrs: { name: 'demo', value: 'b', checked: '' }, innerHTML: 'Text me' },
    { label: 'Off', attrs: { name: 'demo', value: 'a' }, innerHTML: 'Email me' },
    { label: 'Disabled', attrs: { name: 'demo', value: 'c', disabled: '' }, innerHTML: 'Carrier pigeon' },
  ],
  'sui-radio-group': [
    {
      label: 'Plan',
      raw: `<sui-radio-group name="plan" value="pro">
  <sui-radio value="hobby">Hobby</sui-radio>
  <sui-radio value="pro">Pro</sui-radio>
  <sui-radio value="team">Team</sui-radio>
</sui-radio-group>`,
    },
  ],
  'sui-segmented': [
    {
      label: 'Tabs',
      raw: `<sui-segmented value="week">
  <sui-segmented-option value="day">Day</sui-segmented-option>
  <sui-segmented-option value="week">Week</sui-segmented-option>
  <sui-segmented-option value="month">Month</sui-segmented-option>
</sui-segmented>`,
    },
  ],
  'sui-segmented-option': [
    {
      label: 'Inside sui-segmented',
      raw: `<sui-segmented value="week">
  <sui-segmented-option value="day">Day</sui-segmented-option>
  <sui-segmented-option value="week">Week</sui-segmented-option>
  <sui-segmented-option value="month">Month</sui-segmented-option>
</sui-segmented>`,
    },
  ],
  'sui-slider': [
    { label: 'Default', raw: '<div style="width:240px;"><sui-slider value="42" min="0" max="100"></sui-slider></div>' },
    { label: 'Range',   raw: '<div style="width:240px;"><sui-slider value="20,80" min="0" max="100" range></sui-slider></div>' },
  ],
  'sui-textarea': [
    { label: 'Default', attrs: { placeholder: 'Tell us what changed…', rows: '3' } },
    { label: 'Disabled', attrs: { value: 'read only content', disabled: '', rows: '3' } },
  ],
  'sui-file-upload': [
    { label: 'Default', attrs: { accept: '.pdf,.png,.jpg' } },
  ],
  'sui-accordion': [
    {
      label: 'FAQ',
      raw: `<div style="width:320px;text-align:left;"><sui-accordion>
  <sui-accordion-item label="What is SUI?" open>
    A compile-time component framework — templates declare, controllers do.
  </sui-accordion-item>
  <sui-accordion-item label="How is it deployed?">
    Pre-built bundles served by the Ship daemon at /api/assets/.
  </sui-accordion-item>
  <sui-accordion-item label="Is it framework-agnostic?">
    Yes — every primitive is a web component you can drop into any HTML.
  </sui-accordion-item>
</sui-accordion></div>`,
    },
  ],
  'sui-accordion-item': [
    {
      label: 'Inside sui-accordion',
      raw: `<div style="width:320px;text-align:left;"><sui-accordion>
  <sui-accordion-item label="Open by default" open>
    Body content goes here. Slot in any markup.
  </sui-accordion-item>
  <sui-accordion-item label="Closed">
    Click the header to expand.
  </sui-accordion-item>
</sui-accordion></div>`,
    },
  ],
  'sui-collapsible': [
    {
      label: 'Default',
      raw: `<div style="width:320px;text-align:left;"><sui-collapsible open>
  <span slot="trigger" style="font-weight:600;">Advanced options</span>
  <div style="padding:8px 0;">
    Tweak retries, timeouts, and concurrency here.
  </div>
</sui-collapsible></div>`,
    },
  ],
  'sui-stepper': [
    { label: 'Step 2 of 4', attrs: { current: '2', total: '4' } },
    { label: 'Step 3 of 5', attrs: { current: '3', total: '5', size: 'lg', tone: 'accent' } },
  ],
  'sui-pagination': [
    { label: 'Default', attrs: { page: '3', 'page-count': '12' } },
  ],
  'sui-loading': [
    { label: 'Default', attrs: { label: 'Loading…' } },
    { label: 'With spinner', attrs: { variant: 'spinner', label: 'Fetching agents…' } },
  ],
  'sui-error-state': [
    {
      label: 'Generic',
      raw: `<sui-error-state>
  <span slot="title">Something went wrong</span>
  We couldn't load that. Try again, or check the logs.
</sui-error-state>`,
    },
  ],
  'sui-status-bar': [
    {
      label: 'Run summary',
      raw: `<sui-status-bar tone="success">
  <span slot="label">Run #214 — passed</span>
  <span slot="value">12 steps · 1m 04s</span>
</sui-status-bar>`,
    },
  ],
  'sui-divider': [
    { label: 'Horizontal', raw: '<div style="width:280px;"><sui-divider></sui-divider></div>' },
    { label: 'With label', raw: '<div style="width:280px;"><sui-divider label="or"></sui-divider></div>' },
  ],
  'sui-navigation-menu': [
    {
      label: 'Pages',
      raw: `<div style="width:240px;text-align:left;"><sui-navigation-menu>
  <sui-navigation-item href="#" active>Overview</sui-navigation-item>
  <sui-navigation-item href="#">Activity</sui-navigation-item>
  <sui-navigation-item href="#">Settings</sui-navigation-item>
</sui-navigation-menu></div>`,
    },
  ],
  'sui-navigation-item': [
    { label: 'Default', attrs: { href: '#' }, innerHTML: 'Overview' },
    { label: 'Active',  attrs: { href: '#', active: '' }, innerHTML: 'Activity' },
  ],
  'sui-scroll-area': [
    {
      label: 'Default',
      raw: `<div style="width:240px;height:120px;"><sui-scroll-area style="height:120px;width:240px;border:1px solid var(--border);border-radius:6px;">
  <div style="padding:10px 14px;font-family:var(--font-mono);font-size:12px;line-height:1.6;">
    line 01<br>line 02<br>line 03<br>line 04<br>line 05<br>line 06<br>line 07<br>line 08<br>line 09<br>line 10
  </div>
</sui-scroll-area></div>`,
    },
  ],
  'sui-portal': [
    { label: 'Internal', overlay: true, caption: 'Renders its children into the document body. Has no visible footprint until a child paints.' },
  ],
  'sui-error-boundary': [
    { label: 'Wrapper', overlay: true, caption: 'Catches render errors from descendants and shows a fallback. Invisible when healthy.' },
  ],
  'sui-view-chrome': [
    { label: 'Shell', overlay: true, caption: 'Layout container for full-page views (header + body + status bar). Used by the Ship shell, not for inline previews.' },
  ],
  'sui-toast-host': [
    { label: 'Host', overlay: true, caption: 'Mount point for transient toasts. Spawn one with showToast().' },
  ],
  'sui-context-menu': [
    { label: 'Right-click', overlay: true, caption: 'Right-click menu on a target. Appears on contextmenu events.' },
  ],
  'sui-drawer': [
    { label: 'Side drawer', overlay: true, caption: 'Sheet that slides in from an edge. Open via `open` attribute.' },
  ],
  'sui-hover-card': [
    { label: 'On hover', overlay: true, caption: 'Anchored panel that appears on hover. Toggle via `open` attribute.' },
  ],
  'sui-edit-overlay': [
    { label: 'Inline edit', overlay: true, caption: 'Click-to-edit overlay on a target. Appears on activation.' },
  ],
  'sui-auth-dialog': [
    { label: 'Sign-in', overlay: true, caption: 'Auth-flow dialog. Driven by the shell, not used inline.' },
  ],
  'sui-graph-canvas': [
    { label: 'Canvas', overlay: true, caption: 'Force-directed graph canvas. Needs JS-driven node/edge data.' },
  ],

  /* Status / utility primitives without per-variant samples ---------- */
  'sui-mark': [
    { label: 'Highlight', innerHTML: 'highlighted text' },
    { label: 'Accent', attrs: { tone: 'accent' }, innerHTML: 'accent run' },
  ],
  'sui-metachip': [
    { label: 'Pair', attrs: { label: 'env', value: 'prod' } },
    { label: 'Tone', attrs: { label: 'role', value: 'reviewer', tone: 'accent' } },
  ],
  'sui-app-icon': [
    { label: 'Default', attrs: { name: 'studio', size: '40' } },
  ],

  /* Pipeline composites --------------------------------------------- */
  'sui-pipeline-step-def': [
    {
      label: 'Definition row',
      raw: `<sui-pipeline-step-def name="review" step-type="agent" agent="Claude" prompt="Review the PR"></sui-pipeline-step-def>`,
    },
  ],
  'sui-pipeline-definition': [
    {
      label: 'Definition',
      raw: `<sui-pipeline-definition name="release">
  <sui-pipeline-step-def name="collect" step-type="run" agent="git log"></sui-pipeline-step-def>
  <sui-pipeline-step-def name="review" step-type="agent" agent="Claude"></sui-pipeline-step-def>
  <sui-pipeline-step-def name="merge" step-type="merge"></sui-pipeline-step-def>
</sui-pipeline-definition>`,
    },
  ],
  'sui-pipeline-run': [
    {
      label: 'Run #214',
      raw: `<sui-pipeline-run run-id="214" state="running">
  <sui-pipeline-step name="collect" step-type="run" state="completed" agent="git log"></sui-pipeline-step>
  <sui-pipeline-step name="review" step-type="agent" state="running" agent="Claude"></sui-pipeline-step>
  <sui-pipeline-step name="merge" step-type="merge" state="pending"></sui-pipeline-step>
</sui-pipeline-run>`,
    },
  ],
  'sui-harness-blob': [
    { label: 'Claude',   attrs: { harness: 'claude',   size: 'lg' } },
    { label: 'Codex',    attrs: { harness: 'codex',    size: 'lg' } },
    { label: 'Cursor',   attrs: { harness: 'cursor',   size: 'lg' } },
    { label: 'Gemini',   attrs: { harness: 'gemini',   size: 'lg' } },
    { label: 'OpenCode', attrs: { harness: 'opencode', size: 'lg' } },
  ],

  /* Observability primitives missing per-variant samples ------------ */
  'sui-log-stream': [
    {
      label: 'Tail',
      raw: `<div style="width:360px;text-align:left;"><sui-log-stream lines='[{"level":"info","at":"09:14:02","msg":"booted"},{"level":"warn","at":"09:14:09","msg":"throttled (3/min)"},{"level":"error","at":"09:14:21","msg":"upstream 502"}]'></sui-log-stream></div>`,
    },
  ],
  'sui-code': [
    { label: 'Inline', raw: `<sui-code language="ts">const ship = createApp({ persist: true });</sui-code>` },
  ],
  'sui-copy-button': [
    { label: 'Default', attrs: { value: 'echo "ship!"' }, innerHTML: 'Copy command' },
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
