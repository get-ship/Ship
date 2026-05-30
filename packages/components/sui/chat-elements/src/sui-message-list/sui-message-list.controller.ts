import { controller, effect, ref, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-message-list.sui'

interface MessageItem {
  id: string
  content?: unknown
  role?: string
  contentType?: string
  timestamp?: string
  status?: string
}

// How close to the bottom (px) still counts as "pinned" — small scroll
// drifts shouldn't disable auto-scroll.
const BOTTOM_THRESHOLD = 48

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const items = signal<MessageItem[]>([])
    const scrollEl = ref<HTMLElement>(null)
    return { items, scrollEl }
  },

  mount(state, host) {
    // `items` JS-property: `el.items = [...]`.
    const init = (host as unknown as { items?: MessageItem[] }).items
    if (init != null) state.items = init
    Object.defineProperty(host, 'items', {
      configurable: true,
      get: () => state.items,
      set: (v: MessageItem[]) => { state.items = v },
    })

    // Tracks whether the viewport is pinned to the bottom. A user
    // scroll-up flips this false so new items don't yank them down.
    let pinned = true

    const isAtBottom = (el: HTMLElement): boolean =>
      el.scrollHeight - el.scrollTop - el.clientHeight <= BOTTOM_THRESHOLD

    const onScroll = () => {
      const el = state.scrollEl
      if (el) pinned = isAtBottom(el)
    }

    const stopScrollBind = effect(() => {
      const el = state.scrollEl
      if (!el) return
      el.addEventListener('scroll', onScroll, { passive: true })
      return () => el.removeEventListener('scroll', onScroll)
    })

    // On every items change, scroll to bottom when sticky + pinned.
    const stopAutoScroll = effect(() => {
      const count = state.items.length // track items
      void count
      const el = state.scrollEl
      if (!el || !state.stickyBottom) return
      if (!pinned) return
      // Defer to after the keyed list has rendered the new rows.
      queueMicrotask(() => { el.scrollTop = el.scrollHeight })
    })

    return () => {
      stopScrollBind()
      stopAutoScroll()
    }
  },
})
