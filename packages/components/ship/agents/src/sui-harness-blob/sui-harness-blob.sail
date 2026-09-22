<scope>
  props: {
    harness: 'claude' | 'codex' | 'cursor' | 'gemini' | 'opencode' = 'claude'
    size: 'sm' | 'md' | 'lg' = 'md'
    variant: 'tile' | 'glyph' = 'tile'
  }
</scope>
<controller>
import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sui-harness-blob.sail'
import { HARNESSES, type Harness } from './harness-logos'

type StyledHost = { style?: { setProperty: (name: string, value: string) => void } }
type HtmlNode = { innerHTML: string }

export default controller((ctx: ControllerScope<Scope>, host?: Element) => {
    const tileEl = signal<Element | null>(null)
    const captureTile: Action<void> = (node) => {
      tileEl.set(node)
      return { destroy: () => tileEl.set(null) }
    }
    const tileLabel = derived<string>(
      () => (HARNESSES[ctx.harness() as Harness] ?? HARNESSES.claude).label,
    )

    effect(() => {
      const meta = HARNESSES[ctx.harness() as Harness] ?? HARNESSES.claude
      ;(host as StyledHost | undefined)?.style?.setProperty('--blob-hue', meta.hue)
    })

    effect(() => {
      const tile = tileEl() as HtmlNode | null
      if (!tile) return
      const meta = HARNESSES[ctx.harness() as Harness] ?? HARNESSES.claude
      tile.innerHTML = meta.svg
    })

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

<span label={tileLabel()}>
  <span use:captureTile></span>
</span>
<span><slot></slot></span>
