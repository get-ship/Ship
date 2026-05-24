// <sui-blank-canvas> controller — each hint pill dispatches the same
// document-level event the corresponding hotkey would fire, so click +
// key produce identical effects.

import { controller, navigate } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-blank-canvas.sui'

function emit(host: HTMLElement, type: string, detail?: unknown): void {
  host.dispatchEvent(new CustomEvent(type, {
    detail: detail ?? null,
    bubbles: true,
    composed: true,
  }))
}

export default controller({
  scope: (_ctx: ViewScope<Scope>, host?: Element) => {
    const hostEl = host as HTMLElement
    return {
      onPaletteHint: () => emit(hostEl, 'sui-open-command-palette'),
      onStudioHint: () => navigate('/studio'),
      onTerminalHint: () => emit(hostEl, 'sui-toggle-terminal'),
      onHelpHint: () => emit(hostEl, 'sui-open-help'),
    }
  },
})
