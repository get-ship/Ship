/**
 * Bubble toolbar and comment-composer styles for <ship-markdown-editor>.
 * Adopted into the shadow root alongside the editor content styles.
 * Selectors use data-* attrs (sail bans `class` in templates).
 */

export const BUBBLE_CSS = `
[data-md-bubble] {
  position: absolute; z-index: 40;
  display: inline-flex; align-items: center; gap: 0;
  padding: 2px; background: var(--card);
  border: 1px solid var(--border); border-radius: var(--radius-md);
  box-shadow: var(--shadow-md); transform: translateX(-50%);
  opacity: 0; pointer-events: none;
  transform-origin: 50% 100%;
  transition: opacity var(--duration-fast) var(--ease-out),
    transform var(--duration-fast) var(--ease-out);
}
[data-md-bubble][data-open='true'] {
  opacity: 1; pointer-events: auto;
  transform: translate(-50%, -12px);
}
[data-md-bubble] button {
  display: inline-flex; align-items: center; justify-content: center;
  width: 28px; height: 28px; padding: 0; border: 0;
  background: transparent; color: var(--muted-foreground);
  border-radius: var(--radius-sm); cursor: pointer;
  font-family: var(--font-body); font-size: 12px; line-height: 1;
  transition: background-color var(--duration-fast) var(--ease-out),
    color var(--duration-fast) var(--ease-out);
}
[data-md-bubble] button sui-icon { --tone-fg: currentColor; }
[data-md-bubble] button:hover:not(:disabled) { background: color-mix(in oklch, var(--muted) 80%, transparent); color: var(--foreground); }
[data-md-bubble] button:focus-visible { outline: none; box-shadow: 0 0 0 var(--ring-width) color-mix(in oklch, var(--tone, var(--ring-color)) var(--ring-opacity), transparent); color: var(--foreground); }
[data-md-bubble] button[data-is-active='true'] { background: color-mix(in oklch, var(--primary) 18%, var(--muted)); color: var(--foreground); }
[data-md-bubble] [data-md-sep] { width: 1px; height: 18px; background: var(--border); margin: 0 2px; flex-shrink: 0; }
[data-md-bubble] sui-icon::part(svg) { width: 14px; height: 14px; }
@media (prefers-reduced-motion: reduce) { [data-md-bubble] { transition-duration: 1ms; } }

[data-md-composer] {
  position: absolute; z-index: 50;
  width: 320px; max-width: calc(100% - 16px);
  background: var(--card); border: 1px solid var(--border);
  border-radius: var(--radius-lg); box-shadow: var(--shadow-lg);
  overflow: hidden; transform: translate(-50%, 0);
  transform-origin: 50% 0;
  animation: composer-in var(--duration-fast) var(--ease-out);
}
@keyframes composer-in {
  from { opacity: 0; transform: translate(-50%, -4px); }
  to   { opacity: 1; transform: translate(-50%, 0); }
}
[data-md-composer-quote] {
  padding: 8px 12px; font-size: 11px; font-style: italic;
  color: var(--muted-foreground); border-bottom: 1px solid var(--border);
  max-height: 60px; overflow: hidden; line-height: 1.4;
  border-left: 3px solid color-mix(in oklch, var(--primary) 40%, var(--border));
  background: color-mix(in oklch, var(--muted) 35%, transparent);
}
[data-md-composer] textarea {
  display: block; width: 100%; padding: 10px 12px; border: 0;
  background: transparent; color: var(--foreground);
  font-family: var(--font-body); font-size: 13px;
  line-height: 1.5; resize: none; outline: none; min-height: 52px;
}
[data-md-composer] textarea::placeholder { color: var(--muted-foreground); opacity: 0.6; }
[data-md-composer-actions] {
  display: flex; align-items: center; justify-content: flex-end;
  gap: 6px; padding: 6px 8px; border-top: 1px solid var(--border);
  background: color-mix(in oklch, var(--muted) 30%, transparent);
}
[data-md-composer-actions] button {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 5px 10px; border: 0; border-radius: var(--radius-sm);
  font-family: var(--font-body); font-size: 11px; font-weight: 500; cursor: pointer;
  transition: background-color var(--duration-fast) var(--ease-out),
    color var(--duration-fast) var(--ease-out),
    opacity var(--duration-fast) var(--ease-out);
}
[data-md-composer-actions] [data-md-cancel] { background: transparent; color: var(--muted-foreground); }
[data-md-composer-actions] [data-md-cancel]:hover { color: var(--foreground); }
[data-md-composer-actions] button:not([data-md-cancel]) { background: var(--primary); color: var(--primary-foreground); }
[data-md-composer-actions] button:not([data-md-cancel]):hover:not(:disabled) { opacity: 0.92; }
[data-md-composer-actions] button:not([data-md-cancel]):disabled { opacity: 0.4; cursor: default; }
[data-md-composer-actions] kbd { font-family: var(--font-body); font-size: 9px; opacity: 0.75; }
@media (forced-colors: active) {
  [data-md-bubble], [data-md-composer] { border: 1px solid CanvasText; }
}
`

/** Adopt bubble/composer CSS into a light-DOM host (Sail is light DOM). */
export function injectBubbleStyles(host: ParentNode): void {
  if (typeof CSSStyleSheet !== 'undefined' && 'adoptedStyleSheets' in Document.prototype) {
    try {
      const root = host as DocumentOrShadowRoot & ParentNode
      if (root && 'adoptedStyleSheets' in root) {
        const sheet = new CSSStyleSheet()
        sheet.replaceSync(BUBBLE_CSS)
        root.adoptedStyleSheets = [...root.adoptedStyleSheets, sheet]
        return
      }
    } catch { /* fall through to <style> injection */ }
  }
  const style = document.createElement('style')
  style.setAttribute('data-ship-md-bubble', '')
  style.textContent = BUBBLE_CSS
  host.insertBefore(style, host.firstChild)
}
