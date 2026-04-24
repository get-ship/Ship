/**
 * Annotation helpers shared between the React studio overlay and the
 * `<ship-annotation-*>` web components.
 *
 * Pure functions — no DOM globals other than the `Document` / `Element`
 * passed in. Framework-agnostic.
 *
 * The `AnnotationEvent` shape here is the TypeScript mirror of the Rust
 * type `runtime::events::annotation::AnnotationEvent`. JSON field names
 * and enum casing match byte-for-byte.
 */

export type AnnotationKind = 'point' | 'region' | 'action'

export type AnnotationIntent =
  | 'comment'
  | 'request_change'
  | 'regenerate'
  | 'explain'
  | 'approve'
  | 'reject'

export type AuthorKind = 'human' | 'agent'

export interface AnnotationTarget {
  view_id?: string
  file_path?: string
  selector?: string
  rect?: [number, number, number, number]
  viewport?: [number, number]
}

export interface AnnotationSelection {
  kind: string
  text?: string
}

export interface AnnotationEvent {
  id: string
  kind: AnnotationKind
  target: AnnotationTarget
  intent?: AnnotationIntent
  note?: string
  selection?: AnnotationSelection
  created_at: string
  author_kind: AuthorKind
}

// ─────────────────────── Selector + rect queries ───────────────────────

export function getCSSSelector(el: Element): string {
  if (el.id) return `#${el.id}`
  const tag = el.tagName.toLowerCase()
  if (el.className && typeof el.className === 'string') {
    const classes = el.className.trim().split(/\s+/).slice(0, 3).join('.')
    if (classes) return `${tag}.${classes}`
  }
  return tag
}

export function getElementsInRect(
  doc: Document,
  rect: [number, number, number, number],
): string[] {
  const [rx, ry, rw, rh] = rect
  const selectors: string[] = []
  const all = doc.querySelectorAll('*')
  for (const el of all) {
    const r = el.getBoundingClientRect()
    if (r.left >= rx && r.top >= ry && r.right <= rx + rw && r.bottom <= ry + rh) {
      selectors.push(getCSSSelector(el))
    }
    if (selectors.length >= 10) break
  }
  return selectors
}

// ─────────────────────── ULID ───────────────────────

const CROCKFORD = '0123456789ABCDEFGHJKMNPQRSTVWXYZ'

/**
 * Browser-friendly ULID (Crockford base32). The daemon validates with
 * `Ulid::from_string`, which requires exactly this alphabet and length.
 */
export function newUlid(now: number = Date.now()): string {
  let ts = now
  const time = new Array<string>(10)
  for (let i = 9; i >= 0; i--) {
    time[i] = CROCKFORD[ts % 32]!
    ts = Math.floor(ts / 32)
  }
  const rand = new Array<string>(16)
  const cryptoObj =
    typeof globalThis !== 'undefined' ? (globalThis as { crypto?: Crypto }).crypto : undefined
  if (cryptoObj?.getRandomValues) {
    const buf = new Uint8Array(16)
    cryptoObj.getRandomValues(buf)
    for (let i = 0; i < 16; i++) rand[i] = CROCKFORD[buf[i]! % 32]!
  } else {
    for (let i = 0; i < 16; i++) rand[i] = CROCKFORD[Math.floor(Math.random() * 32)]!
  }
  return time.join('') + rand.join('')
}

// ─────────────────────── Event builder ───────────────────────

export interface BuildAnnotationInput {
  viewId?: string
  filePath?: string
  kind: AnnotationKind
  selector?: string
  rect?: [number, number, number, number]
  viewport?: [number, number]
  note?: string
  intent?: AnnotationIntent
  selection?: AnnotationSelection
  authorKind?: AuthorKind
  id?: string
  createdAt?: string
}

/**
 * Assembles an `AnnotationEvent` with the exact JSON shape the daemon
 * expects. Optional fields are elided (not set) when absent so the body
 * matches Rust's `skip_serializing_if = "Option::is_none"`.
 */
export function buildAnnotationEvent(input: BuildAnnotationInput): AnnotationEvent {
  const target: AnnotationTarget = {}
  if (input.viewId !== undefined) target.view_id = input.viewId
  if (input.filePath !== undefined) target.file_path = input.filePath
  if (input.selector !== undefined) target.selector = input.selector
  if (input.rect !== undefined) target.rect = input.rect
  if (input.viewport !== undefined) target.viewport = input.viewport

  const ev: AnnotationEvent = {
    id: input.id ?? newUlid(),
    kind: input.kind,
    target,
    created_at: input.createdAt ?? new Date().toISOString(),
    author_kind: input.authorKind ?? 'human',
  }
  if (input.intent !== undefined) ev.intent = input.intent
  if (input.note !== undefined) ev.note = input.note
  if (input.selection !== undefined) ev.selection = input.selection
  return ev
}
