// Chrome modal queue — renders `chrome.modal.request(opts)` calls as a
// native `<dialog>` and replies to the wrapper via `sui:modal:respond`.
// One dialog at a time; further requests queue and open when the
// previous one closes.
//
// Why native <dialog> rather than <sui-dialog>: this runs from shell2's
// main bundle (not a SUI controller) and must work even when the SUI
// island bundles are still loading. Native <dialog> ships top-layer +
// ::backdrop + focus return + ESC handling out of the box, which is
// exactly the parity sui-dialog was reaching for. The wrapper API is
// shape-compatible — view controllers can build richer prompts atop
// sui-dialog themselves and skip this queue entirely.

export type ModalAction =
  | string
  | {
      label: string
      tone?: 'default' | 'primary' | 'destructive'
      cancel?: boolean
    }

export interface ModalRequestDetail {
  id: number
  title: string
  description?: string
  actions: ModalAction[]
  dismissible?: boolean
}

interface QueuedRequest extends ModalRequestDetail {}

const queue: QueuedRequest[] = []
let active: QueuedRequest | null = null
let dialogEl: HTMLDialogElement | null = null

function ensureDialog(): HTMLDialogElement {
  if (dialogEl) return dialogEl
  const d = document.createElement('dialog')
  d.id = 'ship-modal-queue'
  d.className = 'ship-modal-queue'
  document.body.appendChild(d)
  d.addEventListener('close', () => {
    if (!active) return
    if (d.returnValue) return
    // ESC / backdrop / outright close — resolve with the cancel-flagged
    // action's label when present, empty string otherwise.
    const cancel = active.actions.find(
      (a) => typeof a !== 'string' && a.cancel === true,
    )
    const label = cancel && typeof cancel !== 'string' ? cancel.label : ''
    respond(active.id, label)
    active = null
    drain()
  })
  dialogEl = d
  return d
}

function actionLabel(a: ModalAction): string {
  return typeof a === 'string' ? a : a.label
}

function actionTone(a: ModalAction): string {
  if (typeof a === 'string') return 'default'
  return a.tone ?? 'default'
}

function respond(id: number, action: string): void {
  document.dispatchEvent(
    new CustomEvent('sui:modal:respond', { detail: { id, action } }),
  )
}

function paint(req: QueuedRequest): void {
  const d = ensureDialog()
  d.returnValue = ''
  const title = document.createElement('h2')
  title.className = 'ship-modal-title'
  title.textContent = req.title
  const desc = req.description
    ? Object.assign(document.createElement('p'), {
        className: 'ship-modal-description',
        textContent: req.description,
      })
    : null
  const actions = document.createElement('div')
  actions.className = 'ship-modal-actions'
  req.actions.forEach((a) => {
    const btn = document.createElement('button')
    btn.type = 'button'
    btn.className = `ship-modal-action ship-modal-action--${actionTone(a)}`
    const label = actionLabel(a)
    btn.textContent = label
    btn.addEventListener('click', () => {
      d.returnValue = label
      respond(req.id, label)
      active = null
      d.close(label)
      drain()
    })
    actions.appendChild(btn)
  })
  d.replaceChildren(title, ...(desc ? [desc] : []), actions)
  d.showModal()
  const firstBtn = actions.querySelector('button')
  if (firstBtn instanceof HTMLElement) firstBtn.focus()
}

function drain(): void {
  if (active) return
  const next = queue.shift()
  if (!next) return
  active = next
  paint(next)
}

export function openModalRequest(req: ModalRequestDetail): void {
  queue.push(req)
  drain()
}
