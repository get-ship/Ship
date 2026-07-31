// Pure helpers shared between the agent-detail controller and its
// unit tests. Kept dependency-free so the test file can import them
// without pulling in @ship/sail-runtime or the .sail type stub.

export const HUES = [
  'orange', 'amber', 'red', 'violet', 'blue',
  'teal', 'green', 'pink', 'slate',
] as const

export type Hue = typeof HUES[number]

export function pickHue(id: string): Hue {
  if (!id) return 'slate'
  let h = 0
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) | 0
  return HUES[Math.abs(h) % HUES.length]
}

export function pickInitials(id: string, name?: string): string {
  const src = (id || name || '').trim()
  if (!src) return '?'
  const parts = src.split(/[-_\s]+/).filter(Boolean)
  if (parts.length >= 2) return (parts[0][0] + parts[1][0]).toUpperCase()
  const head = parts[0] || src
  return (head.length >= 2 ? head.slice(0, 2) : head).toUpperCase()
}
