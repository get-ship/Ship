import type { MatchResult, RouteEntry, RouteRegistry } from './types'

interface CompiledPattern {
  /** Tokens: literal segments or { param: name }. */
  segments: Array<{ kind: 'lit'; value: string } | { kind: 'param'; name: string }>
}

function compilePattern(pattern: string): CompiledPattern {
  const segs = pattern.split('/').filter(Boolean)
  return {
    segments: segs.map((s) => {
      const m = /^\[(\w+)\]$/.exec(s)
      if (m) return { kind: 'param', name: m[1]! }
      return { kind: 'lit', value: s }
    }),
  }
}

function tryMatch(
  compiled: CompiledPattern,
  segments: string[],
): Record<string, string> | null {
  if (compiled.segments.length !== segments.length) return null
  const params: Record<string, string> = {}
  for (let i = 0; i < compiled.segments.length; i++) {
    const tok = compiled.segments[i]!
    const seg = segments[i]!
    if (tok.kind === 'lit') {
      if (tok.value !== seg) return null
    } else {
      params[tok.name] = decodeURIComponent(seg)
    }
  }
  return params
}

/**
 * Score a compiled pattern. Lower is more specific (literal segments beat
 * parameters), so static `/agents/list` wins over `/agents/[id]`.
 */
function specificity(compiled: CompiledPattern): number {
  return compiled.segments.reduce((acc, t) => acc + (t.kind === 'param' ? 1 : 0), 0)
}

/**
 * Match a URL pathname against the registry. Returns the best (most-specific)
 * matching route, or `null` for 404. Query strings and hash fragments are
 * ignored — strip them before calling.
 */
export function matchRoute(registry: RouteRegistry, pathname: string): MatchResult | null {
  const segments = pathname.split('/').filter(Boolean)
  let best: { entry: RouteEntry; params: Record<string, string>; score: number } | null = null

  for (const entry of registry.routes) {
    const compiled = compilePattern(entry.manifest.path)
    const params = tryMatch(compiled, segments)
    if (params === null) continue
    const score = specificity(compiled)
    if (best === null || score < best.score) {
      best = { entry, params, score }
    }
  }

  if (best === null) return null
  return { entry: best.entry, params: best.params }
}
