// @ship-set/components — ported primitive registry.
//
// Each component lives under `src/<tag>/<tag>.html.set` (template +
// `<component>` directive + scoped `<style>`) plus `<tag>.ts` (a tiny
// loader that imports the template and registers the custom element).
//
// `registerAll()` is the bundle entry point: the daemon serves a
// pre-built bundle of this module from `/_ship-set/components.js`,
// the shell loads it before any view, and every `<set-*>` tag is
// defined globally before any view template tries to instantiate one.
//
// Authors don't import individual `register*` functions — they just
// drop `<set-status-dot>` (or whatever) into their `.html.set` files.

import { registerSetStatusDot } from './set-status-dot/set-status-dot'
import { registerSetSpinner } from './set-spinner/set-spinner'
import { registerSetBadge } from './set-badge/set-badge'
import { registerSetChip } from './set-chip/set-chip'
import { registerSetCard } from './set-card/set-card'
import { registerSetLink } from './set-link/set-link'
import { registerSetDivider } from './set-divider/set-divider'
import { registerSetSkeleton } from './set-skeleton/set-skeleton'
import { registerSetStat } from './set-stat/set-stat'
import { registerSetMark } from './set-mark/set-mark'
import { registerSetKbd } from './set-kbd/set-kbd'
import { registerSetAvatar } from './set-avatar/set-avatar'
import { registerSetProgress } from './set-progress/set-progress'
import { registerSetButton } from './set-button/set-button'
import { registerSetTabs } from './set-tabs/set-tabs'
import { registerSetTab } from './set-tab/set-tab'
import { registerSetEmptyState } from './set-empty-state/set-empty-state'
import { registerSetInput } from './set-input/set-input'

export { defineComponent } from './define'
export type { DefineComponentOptions } from './define'

export {
  registerSetStatusDot,
  registerSetSpinner,
  registerSetBadge,
  registerSetChip,
  registerSetCard,
  registerSetLink,
  registerSetDivider,
  registerSetSkeleton,
  registerSetStat,
  registerSetMark,
  registerSetKbd,
  registerSetAvatar,
  registerSetProgress,
  registerSetButton,
  registerSetTabs,
  registerSetTab,
  registerSetEmptyState,
  registerSetInput,
}

/** Register every shipped `<set-*>` component on the page document. */
export function registerAll(): void {
  registerSetStatusDot()
  registerSetSpinner()
  registerSetBadge()
  registerSetChip()
  registerSetCard()
  registerSetLink()
  registerSetDivider()
  registerSetSkeleton()
  registerSetStat()
  registerSetMark()
  registerSetKbd()
  registerSetAvatar()
  registerSetProgress()
  registerSetButton()
  registerSetTabs()
  registerSetTab()
  registerSetEmptyState()
  registerSetInput()
}

// Auto-register on module load. The daemon-bundled
// `/_ship-set/components.js` ESM module gets imported from the shell
// HTML; this side-effect is what wires the components in.
if (typeof document !== 'undefined') {
  registerAll()
}
