import { controller, derived, effect, ref } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-alert-dialog.sui'

type DialogEl = HTMLElement & { show(): void; close(): void }

export default controller<Scope>({
  scope: (state, host) => {
    const s = state as unknown as {
      open: Signal<boolean>
      tone: Signal<'default' | 'destructive'>
    }

    const dialogRef = ref<HTMLElement>(null)

    const confirmTone = derived<'default' | 'destructive'>(() =>
      s.tone() === 'destructive' ? 'destructive' : 'default',
    )

    function emit(name: 'sui-confirm' | 'sui-cancel'): void {
      host.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }))
      s.open.set(false)
    }

    function onConfirm(): void { emit('sui-confirm') }
    function onCancel(): void { emit('sui-cancel') }

    return { dialogRef, confirmTone, onConfirm, onCancel }
  },

  mount(state, host) {
    const scope = state as unknown as Scope & {
      open: Signal<boolean>
      dialogRef: Signal<DialogEl | null>
    }

    const openSig = scope.open
    const dialogRef = scope.dialogRef

    // Drive inner sui-dialog open state from our own.
    const stopOpenSync = effect(() => {
      const wantOpen = openSig()
      const dlg = dialogRef() as DialogEl | null
      if (!dlg) return
      if (wantOpen && typeof dlg.show === 'function') dlg.show()
      else if (!wantOpen && typeof dlg.close === 'function') dlg.close()
    })

    // When the inner sui-dialog closes (ESC, backdrop, programmatic),
    // mirror that into our own open=false. The event bubbles through
    // composition to our host.
    const onDialogClose = () => {
      if (openSig()) openSig.set(false)
    }
    host.addEventListener('sui-close', onDialogClose)

    return () => {
      stopOpenSync()
      host.removeEventListener('sui-close', onDialogClose)
    }
  },
})
