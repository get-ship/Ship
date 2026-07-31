<controller>
import { controller, derived, effect } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Signal } from '@ship/sail-runtime'
import type { Scope } from './ship-git-diff-line.sail'

type LineType = 'add' | 'del' | 'context'

export default controller((ctx: ViewScope<Scope>, host) => {

    // Bracket access because `new` is a reserved word — `ctx.new()` is
    // a syntax error, but `ctx['new']()` is legal property access.
    // Similarly `type` is a TS keyword in property position; bracket access required.
    const propBag = ctx as unknown as Record<string, Signal<unknown>>
    const newSig = propBag['new'] as Signal<string>
    const oldSig = propBag['old'] as Signal<string>
    const typeSig = propBag['type'] as Signal<LineType>

    const oldCol = derived<string>(() => oldSig?.() ?? '')
    const newCol = derived<string>(() => newSig?.() ?? '')
    const sigChar = derived<string>(() => {
      const t = typeSig?.()
      return t === 'add' ? '+' : t === 'del' ? '-' : ' '
    })

    function emitComment(e: Event): void {
      e.stopPropagation()
      host?.dispatchEvent(
        new CustomEvent('sail-comment', {
          detail: { old: oldSig?.() ?? '', new: newSig?.() ?? '' },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { oldCol, newCol, sigChar, emitComment }
})
</controller>

<!--
  <ship-git-diff-line> — SUI port of <ship-git-diff-line>.

  Single diff line with old/new gutters, +/-/space signature, the line
  content via the default slot, and a hover-revealed comment button.

  Prop / event / slot parity vs <ship-git-diff-line>
  ─────────────────────────────────────────────────────────────────────────────
  Lit                          SUI                      Notes
  ─────────────────────────────────────────────────────────────────────────────
  type (add|del|context,       type (enum, 'context',   ✓ drives bg tones
    'context')                    reflect)                 via host selectors
  old (string, '')             old (string, '',         ✓ left gutter
                                  reflect)
  new (string, '')             new (string, '',         ✓ right gutter
                                  reflect)
  comment-count (number, 0)    comment-count (number,   ✓ reveals accent
                                  0, reflect)              border + button
                                                           opacity
  ─────────────────────────────────────────────────────────────────────────────
  ship-comment                 sui-comment              renamed per
                                                          convention
  ─────────────────────────────────────────────────────────────────────────────
-->

<!-- <prop name="type" values="add|del|context" default="context" reflect /> -->
<!-- <prop name="old" type="string" default="" reflect /> -->
<!-- <prop name="new" type="string" default="" reflect /> -->
<!-- <prop name="comment-count" type="number" default="0" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<span>{oldCol()}</span>
<span>{newCol()}</span>
<span>{sigChar()}</span>
<span><slot></slot></span>
<button
  type="button"
  aria-label="Comment on this line"
  title="Comment on this line"
  on:click={emitComment}
>+</button>
