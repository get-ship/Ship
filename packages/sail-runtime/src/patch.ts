// Sail patch protocol — the wire format for SSE-driven signal updates.
//
// Patches arrive as JSON over SSE; the runtime parses + applies via
// applyPatch. Every reactive value is path-addressable; ops mutate at
// a JSON path within the signal's value, returning a new value with
// reference equality preserved for unchanged subtrees.
//
// See .artifacts/specs/sail/04-patch-protocol.md for the wire spec.

import { batch, type Signal } from './signal'

export type PatchOp =
  | 'set'
  | 'merge'
  | 'append'
  | 'prepend'
  | 'insert'
  | 'remove'
  | 'move'
  | 'splice'
  | 'inc'

export interface Patch {
  op: PatchOp
  /** JSON path within the signal value. Empty / undefined → root. */
  path?: string
  value?: unknown
  /** insert: target index. */
  index?: number
  /** move: source index. */
  from?: number
  /** move: destination index. */
  to?: number
  /** splice: number of elements to remove. */
  count?: number
  /** inc: numeric increment amount. */
  by?: number
  /** Wire version. Defaults to 1; older clients ignore unknown versions. */
  v?: number
}

/**
 * Apply a patch to a signal. Reads current value, applies the op
 * immutably (copy-on-write along the path), writes back. Subscribers
 * notified once via batched update if multiple patches apply together.
 *
 * Throws on unknown op (forward-compat: future versions may add ops;
 * callers can wrap in try/catch for graceful degradation).
 */
export function applyPatch<T>(sig: Signal<T>, patch: Patch): void {
  const next = applyOp(sig.__peek(), patch)
  sig.set(next as T)
}

/**
 * Apply many patches as one atomic update — subscribers notified once
 * at the end. Useful for multi-field server pushes.
 */
export function applyPatches<T>(sig: Signal<T>, patches: Patch[]): void {
  if (patches.length === 0) return
  batch(() => {
    for (const p of patches) applyPatch(sig, p)
  })
}

// ─── Path tokenization ────────────────────────────────────────────────────────

/**
 * Tokenize a JSON path into segments. Supports `.field`, `[N]` array
 * index, and chains (`a.b[3].c`). Empty path → empty list (root).
 */
export function tokenizePath(path: string): PathToken[] {
  if (!path) return []
  const tokens: PathToken[] = []
  // Match either `field` (identifier) or `[N]` (numeric index).
  const re = /([A-Za-z_$][\w$]*)|\[(\d+)\]/g
  let m: RegExpExecArray | null
  let lastEnd = 0
  while ((m = re.exec(path)) !== null) {
    if (m.index !== lastEnd && lastEnd !== 0 && path[lastEnd] !== '.') {
      // Allow `.` between tokens — anything else means malformed.
      throw new Error(`malformed path at byte ${lastEnd}: ${path}`)
    }
    if (m[1] !== undefined) tokens.push({ kind: 'field', name: m[1] })
    else if (m[2] !== undefined) tokens.push({ kind: 'index', i: Number(m[2]) })
    lastEnd = m.index + m[0].length
  }
  if (lastEnd !== path.length) {
    throw new Error(`malformed path tail: ${path}`)
  }
  return tokens
}

export type PathToken =
  | { kind: 'field'; name: string }
  | { kind: 'index'; i: number }

// ─── Op dispatcher ────────────────────────────────────────────────────────────

function applyOp(value: unknown, p: Patch): unknown {
  const path = tokenizePath(p.path ?? '')
  switch (p.op) {
    case 'set':
      return setAt(value, path, p.value)
    case 'merge':
      return mergeAt(value, path, p.value)
    case 'append':
      return arrayMod(value, path, (arr) => [...arr, p.value])
    case 'prepend':
      return arrayMod(value, path, (arr) => [p.value, ...arr])
    case 'insert': {
      const i = p.index ?? 0
      return arrayMod(value, path, (arr) => {
        const next = arr.slice()
        next.splice(i, 0, p.value)
        return next
      })
    }
    case 'remove':
      return removeAt(value, path)
    case 'move': {
      const from = p.from ?? 0
      const to = p.to ?? 0
      return arrayMod(value, path, (arr) => {
        if (from < 0 || from >= arr.length) return arr
        const next = arr.slice()
        const [item] = next.splice(from, 1)
        next.splice(to, 0, item)
        return next
      })
    }
    case 'splice': {
      const i = p.index ?? 0
      const count = p.count ?? 0
      const inserts = Array.isArray(p.value) ? p.value : []
      return arrayMod(value, path, (arr) => {
        const next = arr.slice()
        next.splice(i, count, ...inserts)
        return next
      })
    }
    case 'inc': {
      const by = p.by ?? 1
      return mapAt(value, path, (v) => (typeof v === 'number' ? v + by : v))
    }
    default:
      throw new Error(`unknown patch op: ${(p as Patch).op}`)
  }
}

// ─── Path-aware immutable helpers ─────────────────────────────────────────────

function setAt(root: unknown, path: PathToken[], value: unknown): unknown {
  if (path.length === 0) return value
  return rebuild(root, path, () => value)
}

function mergeAt(
  root: unknown,
  path: PathToken[],
  partial: unknown,
): unknown {
  return rebuild(root, path, (current) => {
    if (current === null || typeof current !== 'object' || Array.isArray(current)) {
      throw new Error('merge requires an object at path')
    }
    if (partial === null || typeof partial !== 'object' || Array.isArray(partial)) {
      throw new Error('merge value must be an object')
    }
    return { ...(current as object), ...(partial as object) }
  })
}

function removeAt(root: unknown, path: PathToken[]): unknown {
  if (path.length === 0) return undefined
  // Remove the LAST segment of the path from its parent.
  const parentPath = path.slice(0, -1)
  const last = path[path.length - 1]!
  return rebuild(root, parentPath, (parent) => {
    if (last.kind === 'index') {
      if (!Array.isArray(parent)) throw new Error('remove[index] requires array')
      const next = (parent as unknown[]).slice()
      next.splice(last.i, 1)
      return next
    } else {
      if (parent === null || typeof parent !== 'object' || Array.isArray(parent)) {
        throw new Error('remove field requires an object')
      }
      const next = { ...(parent as Record<string, unknown>) }
      delete next[last.name]
      return next
    }
  })
}

function arrayMod(
  root: unknown,
  path: PathToken[],
  mod: (arr: unknown[]) => unknown[],
): unknown {
  return rebuild(root, path, (current) => {
    if (!Array.isArray(current)) throw new Error('expected array at path')
    return mod(current)
  })
}

function mapAt(
  root: unknown,
  path: PathToken[],
  fn: (current: unknown) => unknown,
): unknown {
  return rebuild(root, path, fn)
}

/**
 * Walk `root` along `path`, transform the leaf via `mut`, and rebuild
 * the path with copy-on-write. Returns a new root with reference
 * equality preserved for unchanged subtrees.
 */
function rebuild(
  root: unknown,
  path: PathToken[],
  mut: (current: unknown) => unknown,
): unknown {
  if (path.length === 0) return mut(root)
  const head = path[0]!
  const tail = path.slice(1)

  if (head.kind === 'index') {
    if (!Array.isArray(root)) throw new Error('expected array at path')
    const arr = root as unknown[]
    const next = arr.slice()
    next[head.i] = rebuild(arr[head.i], tail, mut)
    return next
  } else {
    if (root === null || typeof root !== 'object' || Array.isArray(root)) {
      throw new Error('expected object at path')
    }
    const obj = root as Record<string, unknown>
    return { ...obj, [head.name]: rebuild(obj[head.name], tail, mut) }
  }
}
