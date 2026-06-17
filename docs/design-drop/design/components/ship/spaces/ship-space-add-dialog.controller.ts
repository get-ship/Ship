import { controller } from '@ship/sui-runtime'
import type { Scope } from './ship-space-add-dialog.sui'

// No reactive scope state — the input is read on submit. Property-access
// runtime conventions still apply for any future signals.
export default controller<Scope>({
  scope: () => ({}),

  mount(state, host) {
    const readPath = (): string => {
      const input = host.shadowRoot?.querySelector<HTMLInputElement>('.input')
      return (input?.value || '').trim()
    }

    const submit = () => {
      const path = readPath()
      if (!path) return
      host.dispatchEvent(
        new CustomEvent('sui-add', { detail: { path }, bubbles: true, composed: true }),
      )
    }

    const cancel = () => {
      host.dispatchEvent(new CustomEvent('sui-cancel', { bubbles: true, composed: true }))
    }

    const onClick = (e: Event) => {
      for (const node of e.composedPath()) {
        if (!(node instanceof Element)) continue
        if (node.matches('[data-action="submit"]')) return submit()
        if (node.matches('[data-action="cancel"]')) return cancel()
      }
    }

    const onKey = (e: KeyboardEvent) => {
      if (!host.hasAttribute('open')) return
      if (e.key === 'Enter' && (e.target as Element)?.matches?.('.input')) {
        e.preventDefault()
        submit()
      } else if (e.key === 'Escape') {
        cancel()
      }
    }

    host.addEventListener('click', onClick)
    host.addEventListener('keydown', onKey)

    return () => {
      host.removeEventListener('click', onClick)
      host.removeEventListener('keydown', onKey)
    }
  },
})
