import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { PermissionsSection } from './PermissionsSection'
import type { ProfilePermissions } from '@ship/ui'

const meta: Meta<typeof PermissionsSection> = {
  title: 'Agents/PermissionsSection',
  component: PermissionsSection,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof PermissionsSection>

const fullPermissions: ProfilePermissions = {
  preset: 'ship-standard',
  tools_allow: ['Read', 'Grep', 'Glob', 'Bash(git *)'],
  tools_ask: ['Write', 'Edit'],
  tools_deny: ['Bash(rm -rf *)'],
}

const emptyPermissions: ProfilePermissions = {
  tools_allow: [],
  tools_deny: [],
}

/** Standard preset with realistic allow/ask/deny rules. */
export const Standard: Story = {
  args: {
    permissions: fullPermissions,
    activePreset: 'ship-standard',
    onPresetChange: fn(),
    onEdit: fn(),
  },
}

/** Read-only preset with empty rules. */
export const ReadOnly: Story = {
  args: {
    permissions: emptyPermissions,
    activePreset: 'ship-readonly',
    onPresetChange: fn(),
    onEdit: fn(),
  },
}

/** Autonomous preset with broad permissions. */
export const Autonomous: Story = {
  args: {
    permissions: {
      tools_allow: ['Read', 'Write', 'Edit', 'Bash', 'Glob', 'Grep'],
      tools_deny: [],
    },
    activePreset: 'ship-autonomous',
    onPresetChange: fn(),
  },
}
