import { controller, signal, chrome } from '@ship/sui-runtime'
import type { Scope } from './sui-va-compile.sui'

interface ErrorRow {
  id: string
  line: number
  col: number
  severity: 'error' | 'warning'
  message: string
}

interface ErrIn { line?: number; col?: number; severity?: 'error' | 'warning'; message: string }

export default controller<Scope>({
  scope: (state) => {
    const errors = signal<ErrorRow[]>([])
    const s = state as unknown as { view: () => string }
    function onClickRow(err: ErrorRow): void {
      chrome.dock.activate('va-source')
      document.dispatchEvent(new CustomEvent('sui:va:goto-line', {
        detail: { view: s.view(), line: err.line, col: err.col },
        bubbles: true,
      }))
    }
    return { errors, onClickRow }
  },

  mount(state) {
    // sui-runtime mount-state proxy: write via plain assignment.
    const s = state as unknown as {
      errors: ErrorRow[]
      view: () => string
    }

    let counter = 0
    function id(): string { counter += 1; return `err-${counter}` }

    function matches(detailView: string | undefined): boolean {
      const v = s.view()
      if (!v) return true
      if (!detailView) return true
      return detailView === v
    }

    function onErrored(e: Event): void {
      const d = (e as CustomEvent).detail as { view?: string; errors?: ErrIn[] } | undefined
      if (!d || !matches(d.view)) return
      s.errors = (d.errors ?? []).map((er) => ({
        id: id(),
        line: er.line ?? 1,
        col: er.col ?? 1,
        severity: er.severity ?? 'error',
        message: er.message,
      }))
    }
    function onCompiled(e: Event): void {
      const d = (e as CustomEvent).detail as { view?: string } | undefined
      if (!matches(d?.view)) return
      // Successful compile clears the list.
      s.errors = []
    }

    document.addEventListener('sui:view:errored', onErrored)
    document.addEventListener('sui:view:compiled', onCompiled)

    return () => {
      document.removeEventListener('sui:view:errored', onErrored)
      document.removeEventListener('sui:view:compiled', onCompiled)
    }
  },
})
