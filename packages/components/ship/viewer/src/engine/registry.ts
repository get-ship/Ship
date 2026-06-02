// @ship/viewer engine — the default kind→TAG registry table (the eight kinds).
// THIS is the dispatch decision: which renderer TAG a detected content kind
// routes to. Adding a content type is ONE entry here; the <ship-viewer>
// dispatcher template never changes. PURE: no I/O, no `@ship/sui-runtime`.
//
// The pure RESOLUTION engine (createMimeRegistry + the resolveSource walk + MIME
// detection sniffers) lives in @ship/sui-media — this router only owns the table
// and hands it to that engine. The leaf-renderer tags this maps to (image / av /
// csv) are the @ship/sui-media `sui-media-*` components; the markdown / code /
// json / diff / tabular arms map to @ship/sui-primitives composition targets,
// left unchanged. The pdf arm maps to `<ship-pdf>` — the @ship/pdf pack's
// SHIP-tier renderer that wraps a vendored pdf.js engine. PDF is NOT a
// `sui-media-*` leaf (it carries a vendored engine, not a pure renderer), so the
// dispatch table names the ship-tier tag directly.
//
// Authority: build-spec §"The default registry table (the eight kinds)",
// §"Resolution order is FIXED and validated".

import {
  sniffMagicBytes,
  sniffTextBytes,
  type RegistryEntry,
} from '@ship/sui-media/engine'

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
      // pdf maps to `<ship-pdf>` (the @ship/pdf SHIP-tier renderer wrapping a
      // vendored pdf.js engine). It is NOT a `sui-media-*` leaf — the dispatch
      // table names the ship-tier tag directly, and the dispatcher resolves a
      // real, defined element for a pdf source (no fallback/blank screen).
      id: 'pdf',
      kind: 'pdf',
      renderer: 'ship-pdf',
      matchesType: (t) => PDF_TYPES.has(t),
      matchesExt: (e) => PDF_EXT.has(e),
      matchesBytes: (h) => sniffMagicBytes(h) === 'pdf',
    },
    {
      id: 'image',
      kind: 'image',
      renderer: 'sui-media-image',
      matchesType: (t) => IMAGE_TYPES.has(t),
      matchesExt: (e) => IMAGE_EXT.has(e),
      matchesBytes: (h) => sniffMagicBytes(h) === 'image' || sniffTextBytes(h) === 'image',
    },
    {
      id: 'media',
      kind: 'media',
      renderer: 'sui-media-av',
      matchesType: (t) => MEDIA_TYPES.has(t),
      matchesExt: (e) => MEDIA_EXT.has(e),
      matchesBytes: (h) => sniffMagicBytes(h) === 'media',
    },
    {
      // The CSV/TSV renderer composes the committed `sui-table` — the registry
      // names the COMPOSED primitive (`sui-table`); the dispatcher routes the
      // `tabular` kind to `<sui-media-csv>`, which parses then feeds the grid.
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
