/**
 * Styles for <ship-toast-container>.
 *
 * Fixed-position stack host. The six positions (top/bottom × left/center/right)
 * live as attribute selectors so adding a position = one :host([position=…])
 * block. Flex-direction flips between columns so bottom-anchored stacks grow
 * upward and stay visually correct as toasts append.
 *
 * The container is pointer-events: none so clicks fall through the empty
 * gutters between toasts; individual <ship-toast> children re-enable
 * pointer-events on themselves.
 */

import { css } from 'lit'

export const shipToastContainerStyles = css`
  :host {
    position: fixed;
    z-index: var(--z-toast);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    pointer-events: none;
    max-width: min(440px, calc(100vw - 32px));
    /* Nudge inwards from the viewport edge uniformly; per-position blocks
     * below override which sides are anchored. */
    --toast-edge: 16px;
    padding: 0;
    margin: 0;
    box-sizing: border-box;
  }
  :host([hidden]) { display: none; }

  /* Default (unset) matches the legacy ship-toast-container contract:
   * top-right. Newly-appended toasts land at the bottom of the stack. */
  :host(:not([position])),
  :host([position='top-right']) {
    top: var(--toast-edge);
    right: var(--toast-edge);
    bottom: auto;
    left: auto;
    flex-direction: column;
    align-items: flex-end;
  }
  :host([position='bottom-right']) {
    bottom: var(--toast-edge);
    right: var(--toast-edge);
    top: auto;
    left: auto;
    flex-direction: column-reverse;
    align-items: flex-end;
  }
  :host([position='bottom-left']) {
    bottom: var(--toast-edge);
    left: var(--toast-edge);
    top: auto;
    right: auto;
    flex-direction: column-reverse;
    align-items: flex-start;
  }
  :host([position='bottom-center']) {
    bottom: var(--toast-edge);
    left: 50%;
    transform: translateX(-50%);
    top: auto;
    right: auto;
    flex-direction: column-reverse;
    align-items: center;
  }
  :host([position='top-left']) {
    top: var(--toast-edge);
    left: var(--toast-edge);
    bottom: auto;
    right: auto;
    flex-direction: column;
    align-items: flex-start;
  }
  :host([position='top-center']) {
    top: var(--toast-edge);
    left: 50%;
    transform: translateX(-50%);
    bottom: auto;
    right: auto;
    flex-direction: column;
    align-items: center;
  }

  /* Every child toast manages its own pointer-events, but allow pointer
   * interactions on slotted content so hover-to-pause works across the
   * shadow boundary. */
  ::slotted(ship-toast) {
    pointer-events: auto;
    width: auto;
  }
`
