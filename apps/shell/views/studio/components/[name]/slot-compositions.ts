// Default-slot fillers for compound components — atomic ones render
// fine with a humanized label (slotLabel), but compounds need real
// child elements to show their layout (sui-rail empty looks like a
// blank rectangle). Keyed by host tag, shared by the single preview
// and every matrix cell.

// Atomic-component default slot text: the humanized tag, so each
// component shows its own name (sui-button → "Button") instead of a
// repeated "example".
export function slotLabel(tag: string): string {
  const base = tag.replace(/^sui-/, '').replace(/-/g, ' ').trim()
  return base ? base.charAt(0).toUpperCase() + base.slice(1) : 'Preview'
}

export const SLOT_COMPOSITIONS: Record<string, string> = {
  'sui-rail': `
    <sui-rail-section label="Section A">
      <sui-rail-item data-view="agents" active>Agents</sui-rail-item>
      <sui-rail-item data-view="skills">Skills</sui-rail-item>
    </sui-rail-section>
    <sui-rail-section label="Section B">
      <sui-rail-item data-view="git">Git</sui-rail-item>
    </sui-rail-section>
  `,
  'sui-tabs': `
    <sui-tab name="overview" active>Overview</sui-tab>
    <sui-tab name="details">Details</sui-tab>
    <sui-tab name="logs">Logs</sui-tab>
  `,
  'sui-dialog': `
    <span slot="header">Confirm action</span>
    <p>Are you sure you want to continue? This affects 3 items.</p>
    <span slot="footer">
      <sui-button variant="outline">Cancel</sui-button>
      <sui-button variant="primary">Continue</sui-button>
    </span>
  `,
  'sui-alert-dialog': `
    <span slot="title">Delete workspace?</span>
    <p>This permanently removes the workspace and its history.</p>
  `,
  'sui-card': `
    <span slot="header">Card title</span>
    <p>Card body content. Use cards to group related information.</p>
    <span slot="footer">
      <sui-button variant="outline" size="sm">Action</sui-button>
    </span>
  `,
  'sui-accordion': `
    <sui-accordion-item value="one">
      <span slot="trigger">Section one</span>
      <p>First section body.</p>
    </sui-accordion-item>
    <sui-accordion-item value="two">
      <span slot="trigger">Section two</span>
      <p>Second section body.</p>
    </sui-accordion-item>
  `,
  'sui-popover': `
    <sui-button slot="trigger">Open popover</sui-button>
    <p>Popover panel content.</p>
  `,
  'sui-dropdown-menu': `
    <sui-button slot="trigger">Open menu</sui-button>
    <button role="menuitem" data-value="rename">Rename</button>
    <button role="menuitem" data-value="duplicate">Duplicate</button>
    <button role="menuitem" data-value="delete">Delete</button>
  `,
}
