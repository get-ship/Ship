/**
 * Typeahead string-buffer for listbox / menu / tree keyboard search.
 *
 * Collects single-character keystrokes into a buffer that resets after
 * `timeout` ms of inactivity. `match(items, fromIndex)` returns the index
 * of the next item whose label starts with the buffer (case-insensitive),
 * preferring items strictly after `fromIndex` and wrapping. Returns -1 if
 * no match.
 *
 * If only one character has been typed, repeated presses cycle through
 * matches starting with that letter (Listbox APG pattern).
 */

export class Typeahead {
  private buffer = ''
  private lastAt = 0
  constructor(private timeout = 500) {}

  push(key: string): string {
    const now = Date.now()
    if (now - this.lastAt > this.timeout) this.buffer = ''
    this.lastAt = now
    const k = key.toLowerCase()
    // Repeated same single char → keep buffer length 1 so caller cycles.
    if (this.buffer === k) return this.buffer
    this.buffer += k
    return this.buffer
  }

  reset(): void {
    this.buffer = ''
    this.lastAt = 0
  }

  /**
   * Find the next matching item index. Items provide their search label.
   * `fromIndex` is the currently-active index (matches start strictly
   * after it). Buffer of length 1 cycles; longer buffers prefix-match.
   */
  match(labels: readonly string[], fromIndex: number): number {
    const n = labels.length
    if (!this.buffer || n === 0) return -1
    const cycling = this.buffer.length === 1
    const start = cycling ? fromIndex + 1 : 0
    for (let i = 0; i < n; i++) {
      const idx = (start + i) % n
      const label = (labels[idx] ?? '').toLowerCase()
      if (label.startsWith(this.buffer)) return idx
    }
    return -1
  }
}

/** True for printable single-character keys (ignores modifiers, F-keys, etc.). */
export function isTypeaheadKey(e: KeyboardEvent): boolean {
  if (e.ctrlKey || e.altKey || e.metaKey) return false
  return e.key.length === 1 && /\S/.test(e.key)
}
