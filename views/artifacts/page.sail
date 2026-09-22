<controller>
// views/artifacts/controller.ts — host for the @ship/artifacts pack on
// the artifacts view. The pack components are presentational; this
// controller owns the SDK calls and the page-level chrome (drop area,
// upload dialog, detail drawer).
//
// What moved to the pack:
//   - decorate / previewKindFor / bucketFor / AGE_BUCKETS → @ship/artifacts/decorate
//   - mime→viewer dispatch (image / text / json / binary) → <ship-artifact-preview>
//   - inline table markup → <ship-artifact-list>
//   - inline filter inputs → <ship-artifact-filters>
//   - inline action buttons → <ship-artifact-actions>
//   - file upload widget → <ship-artifact-upload>
//
// What stays here:
//   - ship.artifacts.list() live subscription (the list component re-subscribes
//     via its own controller; we keep this for refresh-on-write)
//   - ship.collections.list() subscription (for the upload collection picker)
//   - the detail drawer state + nav-or-open behavior
//   - the upload dialog state + page-level drop
//   - event handlers wiring the pack's `ship-artifact-*` events to SDK calls

import { controller, derived, live, signal } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { ArtifactRecord } from '@ship/sdk-client'
import type { Scope } from './page.sail'

interface ArtifactsListResponse { ok: boolean; data: ArtifactRecord[]; count: number }
interface CollectionsListResponse { items?: Array<{ id: string; name?: string | null }> }
type CollectionRow = { id: string; name?: string | null }

export default controller({
  scope: (ctx: ViewScope) => {
    const { ship } = ctx
    // Shell-injected navigator. Runtime's stock `navigate` only does
    // pushState; the shell's wrapper also triggers mountRoute.
    const navigate = (ctx as { navigate?: (p: string) => void }).navigate
      ?? ((p: string) => { if (typeof window !== 'undefined') window.location.href = p })

    // Live data — the list component owns its own live() subscription;
    // we keep a second one here so the page can force-refresh after
    // upload/promote/delete and the count is shown in the header.
    const reloadNonce = signal(0)
    const artifacts = live<ArtifactsListResponse>(() => {
      reloadNonce()
      return ship.artifacts.list({ limit: 200 }) as never
    })

    const collectionsLive = live(() => ship.collections.list() as never)
    const collectionOptions = derived<CollectionRow[]>(() => {
      const u = collectionsLive() as unknown as { value?: CollectionsListResponse } | undefined
      return u?.value?.items ?? []
    })

    // Detail drawer state.
    const detailItem = signal<ArtifactRecord | null>(null)
    const detailOpen = signal<boolean>(false)
    const detailUrl = derived(() => detailItem() ? `/api/v1/artifacts/${detailItem()!.id}` : '')
    const detailItemJson = derived(() => detailItem() ? JSON.stringify(detailItem()) : '{}')

    // Upload dialog + page-level drop.
    const uploadOpen = signal<boolean>(false)
    const uploadFiles = signal<File[]>([])
    const uploadCollection = signal<string>('')
    const uploadError = signal<string>('')
    const uploading = signal<boolean>(false)
    const dragActive = signal<boolean>(false)

    // JSON projections for the pack components that take JSON-string props.
    const collectionsJson = derived(() => JSON.stringify(collectionOptions()))
    const actorsJson = signal('[]')

    // Header counters. Without filter state on the host (filters are
    // pack-internal for now), the count is the unfiltered list length.
    const totalCount = derived(() => {
      const u = artifacts() as { value?: ArtifactsListResponse } | undefined
      return u?.value?.data?.length ?? 0
    })
    const visibleCount = totalCount

    const submitUpload = async (files: File[]): Promise<void> => {
      if (!files || files.length === 0) {
        uploadError.set('Choose a file to upload.')
        uploadOpen.set(true)
        return
      }
      uploading.set(true)
      uploadError.set('')
      try {
        const file = files[0]!
        const col = uploadCollection().trim()
        const args: { file: File; collection_id?: string } = { file }
        if (col) args.collection_id = col
        await ship.artifacts.upload(args)
        uploadOpen.set(false)
        uploadFiles.set([])
        uploadCollection.set('')
        reloadNonce.set(reloadNonce() + 1)
      } catch (err) {
        uploadError.set((err as Error).message)
        uploadOpen.set(true)
      } finally {
        uploading.set(false)
      }
    }

    return {
      // Live data plumbing (only used for the header counter + refresh nonce)
      artifacts, reloadNonce, collectionsLive,
      // Pack-facing props
      collectionsJson, actorsJson, detailItemJson, detailUrl,
      totalCount, visibleCount,
      // Detail drawer
      detailItem, detailOpen,
      // Upload + drop
      uploadOpen, uploadFiles, uploadCollection, uploadError, uploading, dragActive,
      collectionOptions,
      // Async actions returned from scope so the template can bind them to events
      submitUpload,
      // Bindings the `on:` actions need (no proxy so they bind as plain refs)
      _navigate: navigate,
      _ship: ship as never,
    }
  },

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  on: {
    // s is typed `any` here (not MutableState<Scope>) because the precise Scope type
    // is generated by deck-cli from page.sail and not yet available in this seed copy.
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    refresh(s: any): void {
      s.reloadNonce = (s.reloadNonce as number) + 1
    },

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onRowSelect(s: any, e: { detail: { id: string } }): void {
      const ship = s._ship as { artifacts: { list: (a: unknown) => { data?: ArtifactRecord[] } | undefined } }
      const list = ship.artifacts.list({ limit: 200 })
      const row = list?.data?.find((r) => r.id === e.detail.id) as (ArtifactRecord & { viewer_url?: string | null; viewer_label?: string; name?: string }) | null ?? null
      if (!row) return
      const navigate = s._navigate as (p: string) => void
      if (row.viewer_url) {
        navigate(row.viewer_url)
        return
      }
      s.detailItem = row as ArtifactRecord
      s.detailOpen = true
    },

    onFilterChange(_s: unknown, _e: { detail: { kind: string; value: string } }): void {
      // The pack's filters handle their own state. The host can listen
      // for filter changes here in a future pass when filter state
      // moves to the controller.
    },

    onPromote(_s: unknown, _e: { detail: { id: string } }): void {
      // ship.artifacts.promote is not in the SDK surface yet. Pack
      // documents this gap (KNOWN_GAPS.md). Fallback: refresh so the
      // watch (when wired) picks up the change.
      // TODO: wire ship.artifacts.promote when it lands in the SDK.
    },

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onDelete(s: any, _e: { detail: { id: string } }): void {
      // ship.artifacts.delete is not in the SDK surface yet. Fallback.
      // TODO: wire ship.artifacts.delete when it lands.
      s.detailOpen = false
      s.detailItem = null
    },

    onRename(_s: unknown, _e: { detail: { id: string; name: string } }): void {
      // ship.artifacts.patch is not in the SDK surface yet. Fallback.
      // TODO: wire ship.artifacts.patch when it lands.
    },

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    closeDetail(s: any): void {
      s.detailOpen = false
      s.detailItem = null
    },

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    openUpload(s: any): void {
      s.uploadOpen = true
      s.uploadError = ''
      s.uploadFiles = []
      s.uploadCollection = ''
    },

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    closeUpload(s: any): void {
      if (s.uploading) return
      s.uploadOpen = false
      s.uploadError = ''
      s.uploadFiles = []
    },

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onUploadFiles(s: any, files: File[]): void {
      s.uploadFiles = Array.isArray(files) ? files : []
      s.uploadError = ''
    },

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onUploadCollection(s: any, v: string): void {
      s.uploadCollection = v
    },

    // submitUpload moved to scope (see above) so the template can bind it
    // to a custom event from the pack — async actions in the on: block are
    // not exposed to the template scope.

    // Page-level drop — drop anywhere on the page uploads the file.
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onDragOver(s: any, e: DragEvent): void {
      const dt = e.dataTransfer
      if (!dt || !Array.from(dt.types || []).includes('Files')) return
      e.preventDefault()
      dt.dropEffect = 'copy'
      s.dragActive = true
    },

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onDragLeave(s: any, e: DragEvent): void {
      const t = e.currentTarget as HTMLElement | null
      const related = e.relatedTarget as Node | null
      if (t && related && t.contains(related)) return
      s.dragActive = false
    },

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onDrop(s: any, e: DragEvent): void {
      e.preventDefault()
      s.dragActive = false
      const files = e.dataTransfer?.files
      if (!files || files.length === 0) return
      const first = files[0]
      if (!first) return
      s.uploading = true
      s.uploadError = ''
      const col = (s.uploadCollection as string).trim()
      const ship = s._ship as { artifacts: { upload: (a: { file: File; collection_id?: string }) => Promise<unknown> } }
      const args: { file: File; collection_id?: string } = { file: first }
      if (col) args.collection_id = col
      void ship.artifacts.upload(args)
        .then(() => { s.reloadNonce = (s.reloadNonce as number) + 1 })
        .catch((err: Error) => { s.uploadError = err.message })
        .finally(() => { s.uploading = false })
    },

    // Triggered by the dialog's Upload button. The dialog's submit
    // button fires this; the actual upload call lives in the
    // <ship-artifact-upload> component's own on:ship-artifact-upload
    // event. This is a no-op stub for now — the pack fires the event
    // when the user clicks Upload inside the pack.
    triggerUpload(_s: unknown): void {
      // No-op; the pack owns the upload flow.
    },
  },
})
</controller>

<!--
  views/artifacts/page.sail — the Artifacts browser view.

  Composes the @ship/artifacts pack:
    - <ship-artifact-list>    the live-data table (subscribes to ship.artifacts.list)
    - <ship-artifact-filters>  search + mime/collection/actor/age selects
    - <ship-artifact-actions>  promote / rename / delete verbs
    - <ship-artifact-upload>   drop / file select → host wires ship.artifacts.upload
    - <ship-artifact-preview>  mime-dispatched preview (image / text / json / binary)

  The host (this view) owns the SDK calls for upload / promote / delete / patch.
  The pack components are presentational; they emit `ship-artifact-*` events
  which the page wires to the SDK via the controller.

  Page-level drop (anywhere on the page) is handled by the host so a
  user can drop a file on empty space without opening the upload dialog.
-->
<client name="ship" />

<div
  on:dragenter={onDragOver}
  on:dragover={onDragOver}
  on:dragleave={onDragLeave}
  on:drop={onDrop}>

  <header>
    <div>
      <h1>Artifacts</h1>
      <p>{visibleCount()} of {totalCount()} artifact{totalCount() === 1 ? '' : 's'}</p>
    </div>
    <row gap="2" align="center">
      <button tone="primary" variant="solid" size="sm"
        on:click={openUpload} data-test="upload-button">
        <icon slot="leading" name="upload"></icon>Upload
      </button>
    </row>
  </header>

  <ship-artifact-filters
    collections={collectionsJson()}
    actors={actorsJson()}
    on:ship-artifact-filter={onFilterChange}
  ></ship-artifact-filters>
  <ship-artifact-list
    on:ship-artifact-select={onRowSelect}
  ></ship-artifact-list>

  <if cond={dragActive()}>
    <div data-test="drop-overlay" aria-hidden="true">
      <icon name="upload" size="lg"></icon>
      <span>Drop to upload</span>
    </div>
  </if>

  <dialog open={detailOpen()} size="lg"
    on:sail-close={closeDetail} data-test="detail-drawer">
    <if cond={detailItem()}>
      <span slot="title">{detailItem().name || detailItem().id}</span>
      <span slot="description">{detailItem().mime} · {detailItem().size_label}</span>

      <column gap="4">
        <dl>
          <div>
            <dt>ID</dt>
            <dd><span>{detailItem().id}</span></dd>
          </div>
          <div>
            <dt>Created</dt>
            <dd>{detailItem().created_at}</dd>
          </div>
          <if cond={detailItem().collection_id}>
            <div>
              <dt>Collection</dt>
              <dd><chip tone="muted" variant="soft" size="sm">{detailItem().collection_id}</chip></dd>
            </div>
          </if>
          <if cond={detailItem().scope}>
            <div>
              <dt>Scope</dt>
              <dd><span>{detailItem().scope}</span></dd>
            </div>
          </if>
        </dl>

        <ship-artifact-actions
          record={detailItemJson()}
          on:ship-artifact-promote={onPromote}
          on:ship-artifact-delete={onDelete}
          on:ship-artifact-rename={onRename}
        ></ship-artifact-actions>

        <div data-test="detail-preview">
          <ship-artifact-preview
            record={detailItemJson()}
            url={detailUrl()}
          ></ship-artifact-preview>
        </div>
      </column>

      <div slot="footer">
        <button variant="ghost" on:click={closeDetail}>Close</button>
        <a href={detailUrl()} data-no-router
          download={detailItem().name || detailItem().id}>
          <button tone="primary" variant="solid">
            <icon slot="leading" name="download"></icon>Download
          </button>
        </a>
      </div>
    </if>
  </dialog>

  <dialog open={uploadOpen()} size="md"
    on:sail-close={closeUpload} data-test="upload-dialog">
    <span slot="header">Upload artifact</span>
    <column gap="3">
      <ship-artifact-upload
        collection-id={uploadCollection()}
        disabled={uploading()}
        on:ship-artifact-upload={(e) => submitUpload(e.detail.files)}
      ></ship-artifact-upload>

      <column gap="1">
        <label for="art-upload-col">
          Collection <span>(optional)</span>
        </label>
        <select size="sm" value={uploadCollection()}
          on:sail-change={(e) => onUploadCollection(e.detail.value)}
          disabled={uploading()}
          data-test="upload-collection">
          <option value="">No collection</option>
          <for as="c" of={collectionOptions()} key={c.id}>
            <option value={c.id}>{c.name || c.id}</option>
          </for>
        </select>
      </column>

      <if cond={uploadError()}>
        <div data-test="upload-error">{uploadError()}</div>
      </if>
    </column>
    <div slot="footer">
      <button variant="ghost" on:click={closeUpload}
        disabled={uploading()}>Cancel</button>
      <button tone="primary" variant="solid" on:click={triggerUpload}
        disabled={uploading()}
        loading={uploading()}
        data-test="upload-submit">Upload</button>

    </div>
  </dialog>

</div>
