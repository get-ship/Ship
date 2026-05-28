// Pill controller — bubbles user actions to chrome, parses the
// `queue-json` attr into the structured queue rendered in the
// expanded review panel. Chrome (apps/shell/src/chrome/edit-mode.ts)
// owns the in-memory queue + syncs back via the attr.

import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-edit-mode-pill.sui'

interface QueueItem {
  id: string
  tag: string
  selector: string
  preview: string
  comment?: string
}

function emit(host: HTMLElement, type: string, detail?: unknown): void {
  host.dispatchEvent(new CustomEvent(type, {
    detail: detail ?? null,
    bubbles: true,
    composed: true,
  }))
}

export default controller({
  scope: (state: ViewScope<Scope>, host?: Element) => {
    const hostEl = host as HTMLElement

    const queue = derived<QueueItem[]>(() => {
      try {
        const raw = state.queueJson || '[]'
        const parsed = JSON.parse(raw) as QueueItem[]
        return Array.isArray(parsed) ? parsed : []
      } catch { return [] }
    })

    return {
      queue,
      onExit:            () => emit(hostEl, 'sui:edit-mode:exit'),
      onAnnotateToggle:  () => emit(hostEl, 'sui:annotation-mode:toggle'),
      onSend:            () => { state.expanded = false; emit(hostEl, 'sui:annotations:send') },
      onClearAll:        () => { state.expanded = false; emit(hostEl, 'sui:annotations:clear') },
      onToggleExpand:    () => state.expanded = !state.expanded,
      onRemove:          (id: string) => emit(hostEl, 'sui:annotations:remove', { id }),
      onCommentInput:    (e: Event, id: string) => {
        const ta = e.target as HTMLTextAreaElement | null
        if (!ta) return
        emit(hostEl, 'sui:annotations:comment-update', { id, comment: ta.value })
      },
    }
  },
})
