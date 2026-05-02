// SUI live channel — single EventSource per attached view.
//
// Both `patch-signals` (named LiveSignal value updates from the kernel
// event loop) and `patch-elements` (out-of-band surgical DOM patches
// from the agent-push endpoint) flow on the same SSE stream:
// `/_sui/views/{name}/sse`. This module owns the listener routing.
//
// Selector scoping for `patch-elements` is enforced here by passing
// the view root into the applier — selectors run against
// `root.querySelector`, never `document`, so a patch targeting shell
// chrome finds no match and is dropped with a console warning. See
// `./patch-elements.ts` for the apply path.

import type { LiveState, Signal } from '@ship/ui-runtime'
import { applyPatchElements, type PatchElementsPayload } from './patch-elements'

/** Wire format of a `patch-signals` SSE event's data payload. */
interface PatchSignalsPayload {
  name: string
  value: LiveState<unknown>
}

/**
 * Open one EventSource on `url` and route both event types:
 *
 *  - `patch-signals` → look up `name` in `scope` (filtered through
 *    `liveKeys`) and call `sig.set(value)` on the matching LiveSignal.
 *    Unknown names are silently dropped.
 *  - `patch-elements` → call {@link applyPatchElements} with `root` as
 *    the scoping boundary.
 *
 * Returns the EventSource so the caller can `.close()` it on dispose.
 */
export function openLiveChannel(
  url: string,
  scope: Record<string, Signal<unknown>>,
  liveKeys: Set<string>,
  root: HTMLElement,
): EventSource {
  const es = new EventSource(url)
  es.addEventListener('patch-signals', (event: MessageEvent) => {
    let payload: PatchSignalsPayload
    try {
      payload = JSON.parse(event.data as string) as PatchSignalsPayload
    } catch {
      console.warn(
        '[ship-set/attach] patch-signals: failed to parse event data',
        event.data,
      )
      return
    }
    const { name, value } = payload
    if (!liveKeys.has(name)) return
    const sig = scope[name]
    if (!sig) return
    ;(sig as Signal<LiveState<unknown>>).set(value)
  })
  es.addEventListener('patch-elements', (event: MessageEvent) => {
    let payload: PatchElementsPayload
    try {
      payload = JSON.parse(event.data as string) as PatchElementsPayload
    } catch {
      console.warn(
        '[ship-set/attach] patch-elements: failed to parse event data',
        event.data,
      )
      return
    }
    applyPatchElements(root, payload)
  })
  es.addEventListener('error', () => {
    // EventSource has built-in reconnect; log only to aid debugging.
    console.debug(
      '[ship-set/attach] SSE channel error on',
      url,
      '— will auto-reconnect',
    )
  })
  return es
}
