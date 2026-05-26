import { controller, derived, effect } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-git-diff-line.sui'

type LineType = 'add' | 'del' | 'context'

export default controller({
  scope: (ctx: ViewScope<Scope>, host) => {
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
        new CustomEvent('sui-comment', {
          detail: { old: oldSig?.() ?? '', new: newSig?.() ?? '' },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { oldCol, newCol, sigChar, emitComment }
  },

  mount(state, host) {
    const propBag = state as unknown as Record<string, Signal<unknown>>
    const newSig = propBag['new'] as Signal<string>
    const oldSig = propBag['old'] as Signal<string>
    const typeSig = propBag['type'] as Signal<LineType>

    const stopLabel = effect(() => {
      const t = typeSig?.() ?? 'context'
      const n = newSig?.() ?? ''
      const o = oldSig?.() ?? ''
      const verb = t === 'add' ? 'added' : t === 'del' ? 'removed' : 'context'
      host.setAttribute('aria-label', `line ${n || o} ${verb}`)
    })

    return () => {
      stopLabel()
    }
  },
})
