// Authored controller for the `badge` component proof — confirms a COMPONENT
// (`.sui` + controller) synthesizes through the same compose path as a view.
//
// Phase-1 convention: default-export a `() => scope` factory (see the board
// controller). Component props are a Phase-2 concern (the standalone synthetic
// controller can't model mount-time props yet); this proves a state-only,
// controller-bearing component.
import { signal } from '@ship/sui-runtime'

export default function badgeScope() {
  const count = signal(0)
  const bump = (): void => count.set(count() + 1)
  return { count, bump }
}
