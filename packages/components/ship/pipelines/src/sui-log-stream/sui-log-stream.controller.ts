import { controller, effect, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-log-stream.sui'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const preEl = ref<HTMLPreElement>(null)
    return { preEl }
  },

  mount(state, host) {
    let source: EventSource | null = null
    let buffer = ''
    let stickyBottom = true

    const render = () => {
      const pre = state.preEl as HTMLPreElement | null
      if (!pre) return
      const empty = buffer.length === 0
      if (empty) {
        pre.textContent = '(no output yet)'
        pre.setAttribute('data-empty', '')
      } else {
        pre.textContent = buffer
        pre.removeAttribute('data-empty')
      }
      if (state.autoscroll && stickyBottom) {
        pre.scrollTop = pre.scrollHeight
      }
    }

    const append = (chunk: string) => {
      buffer += chunk
      const cap = state.maxBytes as number
      if (buffer.length > cap) {
        const overflow = buffer.length - cap
        const cut = buffer.indexOf('\n', overflow)
        buffer = buffer.slice(cut >= 0 ? cut + 1 : overflow)
      }
      render()
    }

    const close = () => {
      if (source) {
        source.close()
        source = null
        host.dispatchEvent(
          new CustomEvent('sui-stream-close', { bubbles: true, composed: true }),
        )
      }
    }

    const open = (url: string, eventName: string) => {
      if (!url || source) return
      try {
        source = new EventSource(url, { withCredentials: true })
      } catch {
        host.dispatchEvent(
          new CustomEvent('sui-stream-error', { bubbles: true, composed: true }),
        )
        return
      }
      source.addEventListener(eventName, (ev) => {
        const chunk = (ev as MessageEvent).data ?? ''
        append(chunk)
        host.dispatchEvent(
          new CustomEvent('sui-log-tick', {
            detail: { bytes: chunk.length },
            bubbles: true,
            composed: true,
          }),
        )
      })
      source.onopen = () => {
        host.dispatchEvent(
          new CustomEvent('sui-stream-open', { bubbles: true, composed: true }),
        )
      }
      source.onerror = () => {
        host.dispatchEvent(
          new CustomEvent('sui-stream-error', { bubbles: true, composed: true }),
        )
      }
    }

    const onScroll = (e: Event) => {
      const pre = e.currentTarget as HTMLElement
      stickyBottom = pre.scrollHeight - pre.clientHeight <= pre.scrollTop + 4
    }

    const stopPreBind = effect(() => {
      const pre = state.preEl as HTMLPreElement | null
      if (!pre) return
      pre.addEventListener('scroll', onScroll)
      render()
      return () => pre.removeEventListener('scroll', onScroll)
    })

    // Reconnect when url/event change while active; open/close when active flips.
    const stopActive = effect(() => {
      const url = state.url as string
      const eventName = state.event as string
      const wantActive = state.active as boolean
      if (wantActive) {
        if (source) {
          close()
        }
        open(url, eventName)
      } else {
        close()
      }
    })

    // Expose clear() on the host for callers (parity with lit's clear()).
    type Clearable = HTMLElement & { clear: () => void }
    ;(host as Clearable).clear = () => {
      buffer = ''
      render()
    }

    return () => {
      stopPreBind()
      stopActive()
      close()
    }
  },
})
