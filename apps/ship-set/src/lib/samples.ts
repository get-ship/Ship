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
  'ship-list-item': [
    { label: 'Default', innerHTML: 'Pick a workspace' },
    { label: 'Selected', attrs: { selected: '' }, innerHTML: 'Selected row' },
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
};

export function getShots(tag: string): ComponentShot[] {
  return SHOTS[tag] ?? [{ label: 'Default' }];
}
