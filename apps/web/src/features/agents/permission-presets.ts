// Ship permission presets -- fixed UI constants for one-click permission config.
// These map to the agent schema's `permissions.preset` enum values.

import type { ProfilePermissions } from '@ship/ui'

export interface PermissionPreset {
  id: string
  label: string
  description: string
  permissions: ProfilePermissions
}

export const PERMISSION_PRESETS: PermissionPreset[] = [
  {
    id: 'ship-standard',
    label: 'Standard',
    description: 'Read/write with confirmation on destructive actions',
    permissions: {
      preset: 'ship-standard',
      tools_allow: ['Read', 'Glob', 'Grep'],
      tools_ask: ['Write', 'Edit', 'Bash'],
      tools_deny: [],
    },
  },
  {
    id: 'ship-autonomous',
    label: 'Autonomous',
    description: 'Full access, no confirmations',
    permissions: {
      preset: 'ship-autonomous',
      tools_allow: ['Read', 'Write', 'Edit', 'Bash', 'Glob', 'Grep'],
      tools_ask: [],
      tools_deny: [],
    },
  },
  {
    id: 'ship-readonly',
    label: 'Read Only',
    description: 'Read access only, no file modifications',
    permissions: {
      preset: 'ship-readonly',
      tools_allow: ['Read', 'Glob', 'Grep'],
      tools_ask: [],
      tools_deny: ['Write', 'Edit', 'Bash'],
    },
  },
]

// Built-in tools available across all providers.
export const BUILTIN_TOOLS = [
  'Read', 'Write', 'Edit', 'Bash', 'Glob', 'Grep',
] as const

// Common bash command patterns useful for permission rules.
export const COMMON_BASH_PATTERNS = [
  'bash:npm *',
  'bash:pnpm *',
  'bash:git *',
  'bash:cargo *',
  'bash:docker *',
  'bash:make *',
  'bash:just *',
  'bash:rm *',
  'bash:curl *',
] as const

/** Build tool suggestions from MCP server names. */
export function getMcpToolSuggestions(serverNames: string[]): string[] {
  return serverNames.map((name) => `mcp__${name}__*`)
}
