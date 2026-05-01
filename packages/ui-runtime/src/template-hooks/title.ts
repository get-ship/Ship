// <use-title> — bind document.title to a signal.
//
// Reads `text` inside an `effect` so that when the underlying signal
// changes, `document.title` updates automatically. Restores the
// previous title on unmount.
//
// Example:
//   <use-title bind:text={pageTitle} />
//
// `text` may be a plain string or a signal accessor; both work because
// the effect call re-evaluates the accessor on each dependency change.

import type { TemplateHook, TemplateHookContext } from '../controller'
import { effect } from '../signal'

interface TitleAttrs extends Record<string, unknown> {
  /** Document title — plain string or a signal accessor `() => string`. */
  text: string | (() => string)
}

export const useTitleHook: TemplateHook<TitleAttrs, void> = {
  name: 'title',
  attrs: { text: 'binding' },
  factory: (ctx: TemplateHookContext<TitleAttrs>) => {
    const { text } = ctx.attrs
    if (text === undefined || text === null) return

    const prev = document.title
    const dispose = effect(() => {
      document.title = typeof text === 'function' ? text() : text
    })

    ctx.cleanup(() => {
      dispose()
      document.title = prev
    })
  },
}
