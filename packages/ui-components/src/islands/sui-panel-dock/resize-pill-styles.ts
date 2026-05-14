// Global stylesheet for the resize-height pill teleported by <sui-portal>
// to <body>. The pill lives outside the dock's shadow DOM so it needs
// document-level styles. Singleton install — first instance wins.

const RESIZE_PILL_STYLES = `
.sui-dock-resize-pill {
  position: fixed;
  left: 50%;
  bottom: calc(var(--ship-dock-height, 240px) + 12px);
  transform: translateX(-50%);
  z-index: var(--z-tooltip, 1100);
  padding: 4px 10px;
  border-radius: var(--radius-md, 8px);
  background: var(--popover, var(--card, #fff));
  color: var(--popover-foreground, var(--card-foreground, #111));
  border: 1px solid var(--border, #e5e5e5);
  box-shadow: var(--shadow-md, 0 4px 12px rgba(0, 0, 0, 0.15));
  font-family: var(--mono-font, ui-monospace, monospace);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  pointer-events: none;
  user-select: none;
}
`

let stylesInstalled = false

export function ensureResizePillStyles(): void {
  if (stylesInstalled) return
  if (typeof document === 'undefined') return
  stylesInstalled = true
  const el = document.createElement('style')
  el.setAttribute('data-sui-panel-dock', '')
  el.textContent = RESIZE_PILL_STYLES
  document.head.appendChild(el)
}
