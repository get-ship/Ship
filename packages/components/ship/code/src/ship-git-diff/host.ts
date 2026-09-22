/** Host DOM helpers for the git-diff tree. Kept out of .sail controllers. */

export function setCollapsedOnFiles(host: unknown, collapsed: boolean): void {
  const root = host as { querySelectorAll?: (sel: string) => Iterable<{ setAttribute: (n: string, v: string) => void; removeAttribute: (n: string) => void }> } | null
  const kids = root?.querySelectorAll?.('ship-git-diff-file')
  if (!kids) return
  for (const f of kids) {
    if (collapsed) f.setAttribute('collapsed', '')
    else f.removeAttribute('collapsed')
  }
}

export function dispatchSailComment(host: unknown, detail: { old: string; new: string }): void {
  const el = host as { dispatchEvent?: (e: Event) => boolean } | null
  el?.dispatchEvent?.(
    new CustomEvent('sail-comment', {
      detail,
      bubbles: true,
      composed: true,
    }),
  )
}
