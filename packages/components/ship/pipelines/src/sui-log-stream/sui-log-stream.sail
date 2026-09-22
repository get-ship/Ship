<scope>
  props: {
    url = ''
    event = 'log'
    autoscroll = false
    active = false
    'max-bytes' = 1000000
  }
</scope>
<controller>
import type { Action } from '@ship/sail-runtime'
import type { Scope } from './sui-log-stream.sail'
import { openEventSource } from './sse'

export default controller((ctx: Scope) => {
  let clearBuffer = () => {}

  const capture: Action<void> = (pre) => {
    let closeSource: (() => void) | null = null
    let buffer = ''
    let stickyBottom = true

    const render = () => {
      const empty = buffer.length === 0
      if (empty) {
        pre.textContent = '(no output yet)'
        pre.setAttribute('data-empty', '')
      } else {
        pre.textContent = buffer
        pre.removeAttribute('data-empty')
      }
      if (ctx.autoscroll() && stickyBottom) {
        pre.scrollTop = pre.scrollHeight
      }
    }

    const append = (chunk: string) => {
      buffer += chunk
      const cap = ctx.maxBytes()
      if (buffer.length > cap) {
        const overflow = buffer.length - cap
        const cut = buffer.indexOf('\n', overflow)
        buffer = buffer.slice(cut >= 0 ? cut + 1 : overflow)
      }
      render()
    }

    const close = () => {
      if (closeSource) {
        closeSource()
        closeSource = null
        hostDispatch(pre, 'sail-stream-close', { bubbles: true, composed: true })
      }
    }

    const open = (url: string, eventName: string) => {
      if (!url || closeSource) return
      closeSource = openEventSource(url, eventName, {
        onOpen: () => {
          hostDispatch(pre, 'sail-stream-open', { bubbles: true, composed: true })
        },
        onError: () => {
          hostDispatch(pre, 'sail-stream-error', { bubbles: true, composed: true })
        },
        onChunk: (chunk) => {
          append(chunk)
          hostDispatch(pre, 'sail-log-tick', {
            bubbles: true, composed: true, detail: { bytes: chunk.length },
          })
        },
      })
    }

    const onScroll = (e: { currentTarget?: { scrollHeight: number; clientHeight: number; scrollTop: number } }) => {
      const el = e.currentTarget
      if (!el) return
      stickyBottom = el.scrollHeight - el.clientHeight <= el.scrollTop + 4
    }
    pre.addEventListener('scroll', onScroll)
    render()

    const stopActive = effect(() => {
      const url = ctx.url()
      const eventName = ctx.event()
      const wantActive = ctx.active()
      if (wantActive) {
        if (closeSource) close()
        open(url, eventName)
      } else {
        close()
      }
    })

    clearBuffer = () => {
      buffer = ''
      render()
    }

    return {
      destroy: () => {
        clearBuffer = () => {}
        pre.removeEventListener('scroll', onScroll)
        stopActive()
        close()
      },
    }
  }

  mount((host) => {
    return defineHostProperty(asSailHost(host), 'clear', {
      get: () => () => clearBuffer(),
    })
  })

  return { capture }
})
</controller>

<!--
  <sui-log-stream> — SUI port of <ship-log-stream>.

  Append-only scrollable log pane backed by a Server-Sent Events
  connection. Opens an EventSource against `url`, consumes events of
  the named type (`event`, default `"log"`), and appends each data
  payload as a new line in a monospace scroll container.

  Prop / event parity vs <ship-log-stream>
  ─────────────────────────────────────────────────────────────────────────────
  Lit                          SUI                      Notes
  ─────────────────────────────────────────────────────────────────────────────
  url (string, '')             url (string, '')         ✓ reflects
  event (string, 'log')        event (string, 'log')    ✓ reflects
  autoscroll (bool, false)     autoscroll (bool,        ✓ reflects, sticky-
                                  false, reflect)         bottom autoscroll
  active (bool, false)         active (bool, false,     ✓ reflects, opens
                                  reflect)                /closes EventSource
  max-bytes (number, 1M)       max-bytes (number, 1M)   ✓ rolling cap
  ─────────────────────────────────────────────────────────────────────────────
  ship-stream-open             sui-stream-open          renamed per convention
  ship-stream-error            sui-stream-error         renamed per convention
  ship-stream-close            sui-stream-close         renamed per convention
  ship-log-tick                sui-log-tick             renamed per convention
  ─────────────────────────────────────────────────────────────────────────────
  parts: scroll                parts: scroll            ✓ the <pre>
  ─────────────────────────────────────────────────────────────────────────────
-->

<pre data-empty use:capture>(no output yet)</pre>
