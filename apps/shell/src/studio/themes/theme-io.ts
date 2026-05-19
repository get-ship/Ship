// Theme import/export — pure transforms, no DOM side effects beyond the
// download trigger. Export emits a token-only CSS block (or JSON); import
// parses either a CSS variables block or a JSON `{ "--token": "value" }`
// map and returns the parsed override map.

import type { TokenMap } from './theme-snapshot'

export interface ThemeExport {
  pack: string
  values: TokenMap
}

/** Serialize an override map as a `[data-theme-preview]` CSS block. */
export function toCss(pack: string, values: TokenMap): string {
  const body = Object.entries(values)
    .filter(([, v]) => v)
    .map(([k, v]) => `  ${k}: ${v};`)
    .join('\n')
  return `[data-theme-preview="${pack}"] {\n${body}\n}\n`
}

/** Serialize an override map as JSON. */
export function toJson(pack: string, values: TokenMap): string {
  return JSON.stringify({ pack, values } satisfies ThemeExport, null, 2)
}

/**
 * Parse an imported file. Accepts JSON (`{ pack, values }` or a bare
 * `{ "--token": "value" }` map) or a CSS variables block. Returns the
 * extracted token map, or null if nothing parseable was found.
 */
export function parseImport(text: string): TokenMap | null {
  const trimmed = text.trim()
  if (!trimmed) return null
  if (trimmed.startsWith('{')) {
    try {
      const parsed = JSON.parse(trimmed) as Record<string, unknown>
      const raw = (parsed.values && typeof parsed.values === 'object')
        ? (parsed.values as Record<string, unknown>)
        : parsed
      return coerceMap(raw)
    } catch {
      return null
    }
  }
  return parseCssVars(trimmed)
}

function parseCssVars(css: string): TokenMap | null {
  const out: TokenMap = {}
  const re = /(--[a-z0-9-]+)\s*:\s*([^;]+);/gi
  let m: RegExpExecArray | null
  while ((m = re.exec(css)) !== null) {
    const name = m[1]
    const value = m[2]
    if (name && value) out[name] = value.trim()
  }
  return Object.keys(out).length > 0 ? out : null
}

function coerceMap(raw: Record<string, unknown>): TokenMap | null {
  const out: TokenMap = {}
  for (const [k, v] of Object.entries(raw)) {
    if (k.startsWith('--') && typeof v === 'string') out[k] = v
  }
  return Object.keys(out).length > 0 ? out : null
}

/** Trigger a client-side file download. */
export function downloadFile(name: string, content: string, mime: string): void {
  if (typeof document === 'undefined') return
  const blob = new Blob([content], { type: mime })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = name
  a.click()
  URL.revokeObjectURL(url)
}
