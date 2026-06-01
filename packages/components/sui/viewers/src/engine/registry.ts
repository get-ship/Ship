// sui-viewers engine — the default registry table (the eight kinds) + the pure
// resolution function. Adding a content type is ONE entry here; the dispatcher
// never changes. PURE: no I/O, no `@ship/sui-runtime`.
//
// Authority: build-spec §"The default registry table (the eight kinds)",
// §"Resolution order is FIXED and validated".

import type { RegistryEntry, SourceDescriptor, ViewerSource } from './types'
import { extensionOf, headBytes, normalizeContentType, sniffMagicBytes, sniffTextBytes } from './detect'

const set = (...xs: string[]): ReadonlySet<string> => new Set(xs)

const PDF_TYPES = set('application/pdf')
const PDF_EXT = set('pdf')

const IMAGE_TYPES = set(
  'image/png',
  'image/jpeg',
  'image/gif',
  'image/webp',
  'image/svg+xml',
  'image/avif',
)
const IMAGE_EXT = set('png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'avif')

const MEDIA_TYPES = set(
  'video/mp4',
  'video/webm',
  'audio/mpeg',
  'audio/wav',
  'audio/x-wav',
  'audio/ogg',
  'audio/webm',
  'audio/mp4',
)
const MEDIA_EXT = set('mp4', 'webm', 'mp3', 'wav', 'ogg', 'm4a')

const TABULAR_TYPES = set('text/csv', 'text/tab-separated-values')
const TABULAR_EXT = set('csv', 'tsv')

const MARKDOWN_TYPES = set('text/markdown', 'text/x-markdown')
const MARKDOWN_EXT = set('md', 'markdown')

const CODE_TYPES = set(
  'application/javascript',
  'text/javascript',
  'application/typescript',
  'text/typescript',
  'application/x-typescript',
  'text/x-python',
  'text/x-rust',
  'text/x-go',
  'application/x-sh',
  'text/x-shellscript',
  'application/json5',
)
const CODE_EXT = set('ts', 'tsx', 'js', 'jsx', 'mjs', 'cjs', 'py', 'rs', 'go', 'json5', 'sh', 'bash', 'toml', 'yaml', 'yml')

const JSON_TYPES = set('application/json', 'text/json')
const JSON_EXT = set('json')

const DIFF_TYPES = set('text/x-diff', 'text/x-patch')
const DIFF_EXT = set('diff', 'patch')

/** The default registry table — ordered, most-specific first. The resolver
 *  walks this list per signal tier (content-type, then ext, then bytes). */
export function defaultEntries(): RegistryEntry[] {
  return [
    {
      id: 'pdf',
      kind: 'pdf',
      renderer: 'sui-viewer-pdf',
      matchesType: (t) => PDF_TYPES.has(t),
      matchesExt: (e) => PDF_EXT.has(e),
      matchesBytes: (h) => sniffMagicBytes(h) === 'pdf',
    },
    {
      id: 'image',
      kind: 'image',
      renderer: 'sui-viewer-image',
      matchesType: (t) => IMAGE_TYPES.has(t),
      matchesExt: (e) => IMAGE_EXT.has(e),
      matchesBytes: (h) => sniffMagicBytes(h) === 'image' || sniffTextBytes(h) === 'image',
    },
    {
      id: 'media',
      kind: 'media',
      renderer: 'sui-viewer-media',
      matchesType: (t) => MEDIA_TYPES.has(t),
      matchesExt: (e) => MEDIA_EXT.has(e),
      matchesBytes: (h) => sniffMagicBytes(h) === 'media',
    },
    {
      // The CSV/TSV renderer composes the committed `sui-table` — the registry
      // names the COMPOSED primitive (`sui-table`); the dispatcher routes the
      // `tabular` kind to `<sui-viewer-csv>`, which parses then feeds the grid.
      id: 'tabular',
      kind: 'tabular',
      renderer: 'sui-table',
      matchesType: (t) => TABULAR_TYPES.has(t),
      matchesExt: (e) => TABULAR_EXT.has(e),
    },
    {
      id: 'markdown',
      kind: 'markdown',
      renderer: 'sui-markdown',
      matchesType: (t) => MARKDOWN_TYPES.has(t),
      matchesExt: (e) => MARKDOWN_EXT.has(e),
    },
    {
      // Diff precedes code so `text/x-diff`/`text/x-patch` resolve to the diff
      // renderer before the code arm's `text/x-` catch-all can claim them.
      id: 'diff',
      kind: 'diff',
      renderer: 'sui-diff',
      matchesType: (t) => DIFF_TYPES.has(t),
      matchesExt: (e) => DIFF_EXT.has(e),
      matchesBytes: (h) => sniffTextBytes(h) === 'diff',
    },
    {
      id: 'code',
      kind: 'code',
      renderer: 'sui-code',
      matchesType: (t) => CODE_TYPES.has(t) || (t.startsWith('text/x-') && !DIFF_TYPES.has(t)),
      matchesExt: (e) => CODE_EXT.has(e),
    },
    {
      id: 'json',
      kind: 'json',
      renderer: 'sui-json-view',
      matchesType: (t) => JSON_TYPES.has(t),
      matchesExt: (e) => JSON_EXT.has(e),
    },
  ]
}

const UNSUPPORTED = (fallbackRenderer: string | null, normalizedType: string): SourceDescriptor => ({
  kind: 'unsupported',
  renderer: fallbackRenderer,
  normalizedType,
  reason: fallbackRenderer ? 'fallback' : 'none',
})

/** Resolve a source against an ordered entry list. PURE one-shot — the
 *  precedence is FIXED: content-type, then filename extension, then a
 *  magic-byte sniff of the resolved bytes. First entry that matches at the
 *  highest-confidence available signal wins. */
export function resolveSource(
  entries: readonly RegistryEntry[],
  source: ViewerSource | null,
  fallbackRenderer: string | null,
): SourceDescriptor {
  if (source == null) return UNSUPPORTED(fallbackRenderer, '')

  const normalizedType = normalizeContentType(source.contentType)

  // Tier 1 — content-type (highest confidence).
  if (normalizedType) {
    for (const e of entries) {
      if (e.matchesType?.(normalizedType)) {
        return { kind: e.kind, renderer: e.renderer, normalizedType, reason: 'content-type' }
      }
    }
  }

  // Tier 2 — filename extension.
  const ext = extensionOf(source.filename)
  if (ext) {
    for (const e of entries) {
      if (e.matchesExt?.(ext)) {
        return { kind: e.kind, renderer: e.renderer, normalizedType, reason: 'extension' }
      }
    }
  }

  // Tier 3 — magic-byte sniff of resolved bytes.
  const head = headBytes(source.bytes)
  if (head) {
    for (const e of entries) {
      if (e.matchesBytes?.(head)) {
        return { kind: e.kind, renderer: e.renderer, normalizedType, reason: 'magic-bytes' }
      }
    }
  }

  // Tier 4 — text sniff for SVG/diff carried as `text` (no bytes/type/ext).
  if (source.text) {
    const textHead = new TextEncoder().encode(source.text.slice(0, 512))
    for (const e of entries) {
      if (e.matchesBytes?.(textHead)) {
        return { kind: e.kind, renderer: e.renderer, normalizedType, reason: 'magic-bytes' }
      }
    }
  }

  return UNSUPPORTED(fallbackRenderer, normalizedType)
}
