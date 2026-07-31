<controller>
// <ship-artifact-upload> controller — owns the local file/collection
// input state and emits `ship-artifact-upload { files, collection_id }`.
// No SDK. The host listens and calls ship.artifacts.upload().

import { controller, signal } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-artifact-upload.sail'

export default controller(({ collectionId }: ViewScope<Scope>, host) => {

    const files = signal<File[]>([])
    const draftCollection = signal<string>(collectionId())
    const errorMsg = signal<string>('')

    function onFiles(e: Event): void {
      const detail = (e as CustomEvent<{ files: File[] }>).detail
      files.set(Array.isArray(detail?.files) ? detail.files : [])
      errorMsg.set('')
    }

    function onCollection(e: Event): void {
      const detail = (e as CustomEvent<{ value: string }>).detail
      draftCollection.set(detail?.value ?? '')
    }

    function reset(): void {
      files.set([])
      draftCollection.set(collectionId())
      errorMsg.set('')
    }

    function submit(): void {
      const list = files()
      if (list.length === 0) {
        errorMsg.set('Choose a file to upload.')
        return
      }
      const cid = draftCollection().trim()
      host?.dispatchEvent(
        new CustomEvent('ship-artifact-upload', {
          detail: { files: list, collection_id: cid },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { files, draftCollection, errorMsg, onFiles, onCollection, reset, submit }
})
</controller>

<!--
  <ship-artifact-upload> — @ship/artifacts · drop / select an artifact
  to upload. The host owns the ship.artifacts.upload({ file,
  collection_id }) call (the only SDK mutation in the pack; the rest
  of the verbs are NOT in the SDK yet — see KNOWN_GAPS.md).

  Emits `ship-artifact-upload { files, collection_id }` on submit. No
  SDK here — the host reads the event and calls the SDK.

  Props:
    collection-id   string (current collection)   default ''
    disabled        reflect                       default false
-->
<!-- <prop name="collection-id" type="string" default="" /> -->
<!-- <prop name="disabled" type="bool" default="false" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<column gap="3">
  <div>Upload artifact</div>
  <sail-file-upload
    disabled={disabled()}
    data-test="upload-files"
    on:sail-change={onFiles}></sail-file-upload>

  <row gap="2">
    <label for="upload-collection">Collection</label>
    <input id="upload-collection" size="sm"
      disabled={disabled()}
      value={collectionId()}
      on:sail-input={onCollection}
      data-test="upload-collection-input">
    <span>id: {collectionId() || '—'}</span>
  </row>

  <if cond={errorMsg()}>
    <div role="alert" data-test="upload-error">{errorMsg()}</div>
  </if>

  <row gap="2">
    <button tone="primary" variant="solid" size="sm"
      disabled={disabled() || files().length === 0}
      on:click={submit}
      data-test="upload-submit">Upload</button>
    <button size="sm" variant="ghost"
      disabled={files().length === 0}
      on:click={reset}
      data-test="upload-reset">Clear</button>
  </row>
</column>
