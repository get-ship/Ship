// <sui-shell-top> controller — dispatches the chrome events the shell
// listens for. Palette wiring is a separate job; this just emits.

import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-shell-top.sui'

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
      onCmdKClick: () => emit(hostEl, 'sui-open-command-palette'),
      onThemeClick: () => emit(hostEl, 'sui:theme-picker:open'),
      onNotificationsClick: () => emit(hostEl, 'sui:notifications:open'),
      onAvatarClick: () => emit(hostEl, 'sui:account-menu:open'),
      onBrandClick: (e: MouseEvent) => {
        // Let the anchor's native href="/" route via main.ts's anchor click
        // delegate. We don't preventDefault — the shell's onAnchorClick
        // listener catches the bubbled click and pushes history state.
        void e
      },
    }
  },
})
