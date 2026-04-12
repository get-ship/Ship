/**
 * Shared helpers for Ship web components.
 *
 * Brand tokens are not defined here — they live in `@ship/ui/tokens.css`
 * and are referenced inside component templates as plain `var(--foo)`.
 * The tokens stylesheet is guaranteed loaded by Studio and by the daemon
 * view host, so fallbacks are unnecessary.
 */

export const MONO_STACK =
  "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace"

export function defineOnce(name: string, ctor: CustomElementConstructor): void {
  if (typeof customElements === 'undefined') return
  if (!customElements.get(name)) customElements.define(name, ctor)
}
