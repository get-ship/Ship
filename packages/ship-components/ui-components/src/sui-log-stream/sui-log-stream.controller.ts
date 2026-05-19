import { controller, effect, ref } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-log-stream.sui'

export default controller<Scope>({
  scope: () => {
    const preEl = ref<HTMLPreElement>(null)
    return { preEl }
  },

  mount(state, host) {
    const scope = state as unknown as {
      url: Signal<string>
      event: Signal<string>
      autoscroll: Signal<boolean>
      active: Signal<boolean>
      maxBytes: Signal<number>
      preEl: Signal<HTMLPreElement | null>
    }

    let source: EventSource | null = null
    let buffer = ''
    let stickyBottom = true

    const render = () => {
      const pre = scope.preEl()
      if (!pre) return
      const empty = buffer.length === 0
      if (empty) {
        pre.textContent = '(no output yet)'
        pre.setAttribute('data-empty', '')
      } else {
        pre.textContent = buffer
        pre.removeAttribute('data-empty')
      }
      if (scope.autoscroll() && stickyBottom) {
        pre.scrollTop = pre.scrollHeight
      }
    }

    const append = (chunk: string) => {
      buffer += chunk
      const cap = scope.maxBytes()
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

    const open = () => {
      const url = scope.url()
      if (!url || source) return
      try {
        source = new EventSource(url, { withCredentials: true })
      } catch {
        host.dispatchEvent(
          new CustomEvent('sui-stream-error', { bubbles: true, composed: true }),
        )
        return
      }
      source.addEventListener(scope.event(), (ev) => {
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
      const pre = scope.preEl()
      if (!pre) return
      pre.addEventListener('scroll', onScroll)
      render()
      return () => pre.removeEventListener('scroll', onScroll)
    })

    // Reconnect when url/event change while active; open/close when active flips.
    const stopActive = effect(() => {
      scope.url()
      scope.event()
      const wantActive = scope.active()
      if (wantActive) {
        if (source) {
          close()
        }
        open()
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
