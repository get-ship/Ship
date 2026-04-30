// Template hooks — `<use-*>` elements ship-set ships built-in.
//
// The runtime registers each hook by name at module load. Authors
// don't import these directly — they appear in `.html.set` templates
// as `<use-name>` elements and the dom hydrate flow looks them up
// in the registry.
//
// Three hooks ship in v1: `<use-kbd>`, `<use-click-outside>`,
// `<use-shell>`. See docs/ship-set-framework/references/docs/hooks.md.

import { registerTemplateHook } from './registry'
import { useKbdHook } from './kbd'
import { useClickOutsideHook } from './click-outside'
import { useShellHook } from './shell'

export {
  registerTemplateHook,
  getTemplateHook,
  clearRegistry,
  listRegisteredHooks,
} from './registry'

export type { KbdShortcut } from './kbd'
export type { ShellContext } from './shell'

// Built-in hooks — registered eagerly at module load.
registerTemplateHook(useKbdHook)
registerTemplateHook(useClickOutsideHook)
registerTemplateHook(useShellHook)
