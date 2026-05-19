// <sui-shell-bottom> controller — bubbles click intents the shell listens
// for. Glance-only; no internal state.

import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-shell-bottom.sui'

function emit(host: HTMLElement, type: string, detail?: unknown): void {
  host.dispatchEvent(new CustomEvent(type, {
    detail: detail ?? null,
    bubbles: true,
    composed: true,
  }))
}

export default controller<Scope>({
  scope: (_state, host) => {
    const hostEl = host as HTMLElement
    return {
      onDotClick: () => emit(hostEl, 'sui:dock:toggle'),
      onTerminalClick: () => emit(hostEl, 'sui-toggle-terminal'),
      onDevtoolsClick: () => emit(hostEl, 'sui:dock:toggle'),
      onAnnotateClick: () => emit(hostEl, 'sui:annotation-mode:toggle'),
      onThemeClick: () => emit(hostEl, 'sui:theme-picker:open'),
    }
  },
})
