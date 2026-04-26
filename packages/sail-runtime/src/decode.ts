// Prop attribute decoders. The compiler emits a decoder per prop based
// on its declared type. Strings come off the DOM as `string | null`;
// these turn that into the typed value the script expects.
//
// See docs/sail/01-language-spec.md § Prop types.

export type Decoder<T> = (raw: string | null) => T

/** Identity decoder — pass through. `null` → `''` for missing attrs. */
export const decodeString: Decoder<string> = (raw) => raw ?? ''

/** Number decoder. Empty/missing → `NaN`; let the author guard. */
export const decodeNumber: Decoder<number> = (raw) => {
  if (raw === null || raw === '') return NaN
  return Number(raw)
}

/**
 * Boolean decoder — HTML attribute presence semantics.
 *
 * - Missing attribute (`null`) → `false`
 * - Present with empty value (`""`) → `true` (e.g. `<x disabled>`)
 * - `"true"` / `"false"` (case-insensitive) → respective boolean
 * - Anything else present → `true`
 */
export const decodeBoolean: Decoder<boolean> = (raw) => {
  if (raw === null) return false
  const v = raw.toLowerCase()
  if (v === 'false') return false
  return true
}

/** JSON decoder for object/array props. Returns fallback on parse error. */
export function decodeJson<T>(fallback: T): Decoder<T> {
  return (raw) => {
    if (raw === null || raw === '') return fallback
    try {
      return JSON.parse(raw) as T
    } catch {
      return fallback
    }
  }
}

/** Enum decoder — only allow listed values; otherwise return the default. */
export function decodeEnum<T extends string>(allowed: readonly T[], fallback: T): Decoder<T> {
  const set = new Set<string>(allowed)
  return (raw) => (raw !== null && set.has(raw) ? (raw as T) : fallback)
}
