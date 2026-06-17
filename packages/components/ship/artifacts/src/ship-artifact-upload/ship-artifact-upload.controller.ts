// <ship-artifact-upload> controller — owns the local file/collection
// input state and emits `ship-artifact-upload { files, collection_id }`.
// No SDK. The host listens and calls ship.artifacts.upload().

import { controller, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-artifact-upload.sui'

export default controller({
  scope: ({ collectionId }: ViewScope<Scope>, host) => {
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
  },
})
