<scope>
  props: {
    record = '{}'
  }
</scope>
<controller>
// <ship-artifact-actions> controller — owns the rename input + delete
// confirm dialog state and emits the three action events. No SDK, no
// live data. The host wires the events to the future Patch / Promote /
// Delete SDK ops (see KNOWN_GAPS.md).

import type { Scope } from './ship-artifact-actions.sail'

interface RecordProp {
  id: string
  name: string
}

type FieldEv = { detail?: { value?: string }; target?: { value?: string } }

function parseRecord(raw: string): RecordProp {
  try {
    const o = JSON.parse(raw) as Partial<RecordProp>
    return { id: typeof o.id === 'string' ? o.id : '', name: typeof o.name === 'string' ? o.name : '' }
  } catch {
    return { id: '', name: '' }
  }
}

function fieldValue(e: FieldEv): string {
  return e.detail?.value ?? e.target?.value ?? ''
}

export default controller((ctx: Scope, host?: Element) => {
  const renaming = signal<boolean>(false)
  const renameDraft = signal<string>('')
  const deleteOpen = signal<boolean>(false)

  // `recordId` / `displayName` are template-facing derived values
  // (button disable + name pill). They name a stable concept the
  // template binds by, so they're kept as named callables.
  const currentRecord = (): RecordProp => parseRecord(ctx.record())
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

  function onRenameInput(e: FieldEv): void {
    renameDraft.set(fieldValue(e))
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
    onPromote, onRenameStart, onRenameCancel, onRenameSubmit, onRenameInput, onRenameKey,
    openDelete, closeDelete, confirmDelete,
  }
})
</controller>

<!--
  <ship-artifact-actions> — @ship/artifacts · the three action verbs for a
  single artifact row. Promote (primary) · Rename (ghost) · Delete
  (destructive, confirm via dialog).

  Pure presentational. No SDK. The host owns the SDK call:
    - `ship-artifact-promote { id }`
    - `ship-artifact-rename  { id, name }`
    - `ship-artifact-delete  { id }`

  Props:
    record    JSON of { id, name, ... }      default '{}'
-->

<div>
  <span data-test="actions-name">{displayName()}</span>

  <button tone="primary" variant="solid" size="sm"
    disabled={!recordId()}
    on:click={onPromote}
    data-test="action-promote">Promote</button>

  <button variant="ghost" size="sm"
    disabled={!recordId()}
    on:click={onRenameStart}
    data-test="action-rename">Rename</button>

  <button tone="destructive" variant="solid" size="sm"
    disabled={!recordId()}
    on:click={openDelete}
    data-test="action-delete">Delete</button>

  <if {renaming()}>
    <span data-test="rename-form">
      <input
        type="text"
        value={renameDraft()}
        placeholder="New name"
        aria-label="New artifact name"
        on:input={onRenameInput}
        on:keydown={onRenameKey} />
      <button size="sm" on:click={onRenameSubmit} data-test="rename-submit">Save</button>
      <button size="sm" variant="ghost" on:click={onRenameCancel} data-test="rename-cancel">Cancel</button>
    </span>
  </if>
</div>

<dialog
  open={deleteOpen()}
  size="sm"
  on-close={closeDelete}
  on-dismiss={closeDelete}
  data-test="delete-dialog">
  <header>Delete artifact?</header>
  <section>
    <p>This permanently removes <strong>{displayName()}</strong>. This action cannot be undone.</p>
  </section>
  <footer>
    <row gap="2" justify="end">
      <button variant="ghost" on:click={closeDelete} data-test="delete-cancel">Cancel</button>
      <button tone="destructive" variant="solid" on:click={confirmDelete} data-test="delete-confirm">Delete</button>
    </row>
  </footer>
</dialog>
