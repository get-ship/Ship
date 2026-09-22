// Re-export the mime-registry engine so the <ship-viewer> controller stays
// package-local (relative imports only).
export {
  createMimeRegistry,
  extensionOf,
  type ViewerSource,
} from '@ship/sail-media/engine'

import type { ViewerSource } from '@ship/sail-media/engine'

export function pdfUrlOf(s: ViewerSource | null): string {
  if (!s) return ''
  if (s.url) return s.url
  if (s.bytes) {
    const part = (s.bytes instanceof Uint8Array ? s.bytes : new Uint8Array(s.bytes)) as BlobPart
    return URL.createObjectURL(new Blob([part], { type: 'application/pdf' }))
  }
  return ''
}
