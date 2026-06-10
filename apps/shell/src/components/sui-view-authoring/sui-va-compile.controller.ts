import { controller, signal, chrome } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-va-compile.sui'

interface ErrorRow {
  id: string
  line: number
  col: number
  severity: 'error' | 'warning'
  message: string
}

interface ErrIn { line?: number; col?: number; severity?: 'error' | 'warning'; message: string }

export default controller({
  scope: (state: ViewScope<Scope>) => {
    const errors = signal<ErrorRow[]>([])
    function onClickRow(err: ErrorRow): void {
      chrome.dock.activate('va-source')
      document.dispatchEvent(new CustomEvent('sui:va:goto-line', {
        detail: { view: state.view, line: err.line, col: err.col },
        bubbles: true,
      }))
    }
    return { errors, onClickRow }
  },

  mount(state) {
    // sui-runtime mount-state proxy: `state.view` reads the unwrapped signal
    // value; `state.errors = …` writes through the set-trap.
    let counter = 0
    function id(): string { counter += 1; return `err-${counter}` }

    function matches(detailView: string | undefined): boolean {
      const v = state.view
      if (!v) return true
      if (!detailView) return true
      return detailView === v
    }

    function onErrored(e: Event): void {
      const d = (e as CustomEvent).detail as { view?: string; errors?: ErrIn[] } | undefined
      if (!d || !matches(d.view)) return
      state.errors = (d.errors ?? []).map((er) => ({
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
      state.errors = []
    }

    document.addEventListener('sui:view:errored', onErrored)
    document.addEventListener('sui:view:compiled', onCompiled)

    return () => {
      document.removeEventListener('sui:view:errored', onErrored)
      document.removeEventListener('sui:view:compiled', onCompiled)
    }
  },
})
