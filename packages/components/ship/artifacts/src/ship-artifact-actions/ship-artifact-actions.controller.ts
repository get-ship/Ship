// <ship-artifact-actions> controller — owns the rename input + delete
// confirm dialog state and emits the three action events. No SDK, no
// live data. The host wires the events to the future Patch / Promote /
// Delete SDK ops (see KNOWN_GAPS.md).

import { controller, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-artifact-actions.sui'

interface RecordProp {
  id: string
  name: string
}

function parseRecord(raw: string): RecordProp {
  try {
    const o = JSON.parse(raw) as Partial<RecordProp>
    return { id: typeof o.id === 'string' ? o.id : '', name: typeof o.name === 'string' ? o.name : '' }
  } catch {
    return { id: '', name: '' }
  }
}

export default controller({
  scope: ({ record }: ViewScope<Scope>, host) => {
    const renaming = signal<boolean>(false)
    const renameDraft = signal<string>('')
    const deleteOpen = signal<boolean>(false)

    // `recordId` / `displayName` are template-facing derived values
    // (button disable + name pill). They name a stable concept the
    // template binds by, so they're kept as named callables.
    const currentRecord = (): RecordProp => parseRecord(record())
    const recordId = (): string => currentRecord().id
    const displayName = (): string => currentRecord().name || currentRecord().id

    function emitEvent(name: string, detail: Record<string, unknown>): void {
      host?.dispatchEvent(new CustomEvent(name, { detail, bubbles: true, composed: true }))
    }

    function onPromote(): void {
      const id = recordId()
      if (id) emitEvent('ship-artifact-promote', { id })
    }

    function onRenameStart(): void {
      const r = currentRecord()
      if (!r.id) return
      renameDraft.set(r.name)
      renaming.set(true)
    }

    function onRenameCancel(): void {
      renaming.set(false)
      renameDraft.set('')
    }

    function onRenameSubmit(): void {
      const id = recordId()
      const name = renameDraft().trim()
      if (id && name) emitEvent('ship-artifact-rename', { id, name })
      renaming.set(false)
      renameDraft.set('')
    }

    function onRenameKey(e: Event): void {
      const k = (e as KeyboardEvent).key
      if (k === 'Enter') onRenameSubmit()
      else if (k === 'Escape') onRenameCancel()
    }

    function openDelete(): void {
      if (recordId()) deleteOpen.set(true)
    }

    function closeDelete(): void {
      deleteOpen.set(false)
    }

    function confirmDelete(): void {
      const id = recordId()
      if (id) emitEvent('ship-artifact-delete', { id })
      deleteOpen.set(false)
    }

    return {
      renaming, renameDraft, deleteOpen,
      recordId, displayName,
      onPromote, onRenameStart, onRenameCancel, onRenameSubmit, onRenameKey,
      openDelete, closeDelete, confirmDelete,
    }
  },
})
