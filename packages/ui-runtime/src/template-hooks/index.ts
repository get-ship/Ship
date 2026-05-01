// Template hooks — `<use-*>` elements ship-set ships built-in.
//
// The runtime registers each hook by name at module load. Authors
// don't import these directly — they appear in `.html.set` templates
// as `<use-name>` elements and the dom hydrate flow looks them up
// in the registry.
//
// Eight hooks ship: `<use-kbd>`, `<use-click-outside>`, `<use-shell>`,
// `<use-focus-trap>`, `<use-clipboard>`, `<use-title>`,
// `<use-scroll-anchor>`, `<use-intersection>`.
// See docs/ship-ui/references/docs/hooks.md.

import { registerTemplateHook } from './registry'
import { useKbdHook } from './kbd'
import { useClickOutsideHook } from './click-outside'
import { useShellHook } from './shell'
import { useFocusTrapHook } from './focus-trap'
import { useClipboardHook } from './clipboard'
import { useTitleHook } from './title'
import { useScrollAnchorHook } from './scroll-anchor'
import { useIntersectionHook } from './intersection'

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
registerTemplateHook(useFocusTrapHook)
registerTemplateHook(useClipboardHook)
registerTemplateHook(useTitleHook)
registerTemplateHook(useScrollAnchorHook)
registerTemplateHook(useIntersectionHook)
