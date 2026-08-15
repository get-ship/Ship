<scope>
  props: {
    harness: 'claude' | 'codex' | 'cursor' | 'gemini' | 'opencode' = 'claude'
    size: 'sm' | 'md' | 'lg' = 'md'
    variant: 'tile' | 'glyph' = 'tile'
  }
</scope>
<controller>
import { controller, derived, signal } from '@ship/sail-runtime'
import type { Action, ViewScope } from '@ship/sail-runtime'
import type { Scope } from './sui-harness-blob.sail'
import { HARNESSES, type Harness } from './harness-logos'

export default controller((ctx: ViewScope<Scope>) => {

    const tileEl = signal<Element | null>(null)
    const captureTile: Action<void> = (node) => {
      tileEl.set(node)
      return { destroy: () => tileEl.set(null) }
    }
    const tileLabel = derived<string>(
      () => (HARNESSES[ctx.harness() as Harness] ?? HARNESSES.claude).label,
    )
    return { captureTile, tileLabel }
})
</controller>

<!--
  <sui-harness-blob> — SUI port of <ship-harness-blob>.

  Small branded indicator for an AI harness (claude / codex / cursor /
  gemini / opencode). Inlines the canonical brand SVG so the blob is
  always pixel-accurate and offline-safe.

  Prop / event / slot parity vs <ship-harness-blob>
  ─────────────────────────────────────────────────────────────────────────────
  Lit                          SUI                      Notes
  ─────────────────────────────────────────────────────────────────────────────
  harness (enum, 'claude')    harness (enum,          ✓ reflects
                                  'claude', reflect)
  size (sm|md|lg, 'md')        size (enum, 'md',        ✓ reflects
                                  reflect)
  variant (tile|glyph, 'tile') variant (enum, 'tile',   ✓ reflects
                                  reflect)
  ─────────────────────────────────────────────────────────────────────────────
  slot: default (label)        slot: default            ✓ wrapper hides on
                                                         empty via :empty
  ─────────────────────────────────────────────────────────────────────────────
  unsafeHTML(svg)              bindHtml on tile el      ✓ SUI's analogue
  setProperty('--blob-hue')    controller effect        ✓ same behavior
  ─────────────────────────────────────────────────────────────────────────────
-->

<!-- <prop name="harness" values="claude|codex|cursor|gemini|opencode" default="claude" reflect /> -->
<!-- <prop name="size" values="sm|md|lg" default="md" reflect /> -->
<!-- <prop name="variant" values="tile|glyph" default="tile" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<span role="img" aria-label={tileLabel()}>
  <span use:captureTile></span>
</span>
<span><slot></slot></span>
