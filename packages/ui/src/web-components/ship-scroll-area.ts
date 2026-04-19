/**
 * <ship-scroll-area>
 *
 * Custom-scrollbar viewport. Provides a styled scrollable region with
 * auto-hiding scrollbars that appear on hover or active scroll. Supports
 * both vertical and horizontal scrolling.
 *
 * Attributes:
 *   - orientation: vertical | horizontal | both (default: vertical)
 *   - size: sm | md | lg (scrollbar thumb width channel, default: md)
 *
 * Slots:
 *   - (default) — scrollable content.
 *
 * Parts:
 *   - viewport — the scrollable container.
 *   - scrollbar-v — vertical scrollbar track.
 *   - scrollbar-h — horizontal scrollbar track.
 */

import { LitElement, css, html } from 'lit'
import { reducedMotion } from './util'

export class ShipScrollArea extends LitElement {
  static styles = [reducedMotion, css`
    :host {
      display: block;
      position: relative;
      overflow: hidden;
    }
    :host([hidden]) { display: none; }

    /* Size channel for scrollbar thumb width */
    :host(:not([size])), :host([size='md']) { --sb-size: 8px; }
    :host([size='sm']) { --sb-size: 6px; }
    :host([size='lg']) { --sb-size: 10px; }

    .viewport {
      width: 100%;
      height: 100%;
      overflow: hidden;
      scrollbar-width: none;         /* Firefox */
      -ms-overflow-style: none;       /* IE / Edge */
    }
    .viewport::-webkit-scrollbar { display: none; }

    :host([orientation='vertical']) .viewport,
    :host(:not([orientation])) .viewport {
      overflow-y: auto;
    }
    :host([orientation='horizontal']) .viewport {
      overflow-x: auto;
    }
    :host([orientation='both']) .viewport {
      overflow: auto;
    }

    /* Scrollbar tracks — absolute positioned overlays */
    .scrollbar {
      position: absolute;
      opacity: 0;
      transition: opacity var(--duration-medium) var(--ease-out);
      z-index: 1;
    }
    :host(:hover) .scrollbar,
    :host(:focus-within) .scrollbar,
    .scrollbar.active {
      opacity: 1;
    }

    .scrollbar-v {
      top: 0; right: 2px; bottom: 0;
      width: var(--sb-size);
    }
    .scrollbar-h {
      left: 0; right: 0; bottom: 2px;
      height: var(--sb-size);
    }

    /* Thumb */
    .thumb {
      position: absolute;
      border-radius: var(--radius-full);
      background: color-mix(in oklch, var(--foreground) 30%, transparent);
      transition: background-color var(--duration-fast) var(--ease-out);
    }
    .thumb:hover {
      background: color-mix(in oklch, var(--foreground) 50%, transparent);
    }
    .scrollbar-v .thumb {
      width: 100%;
      min-height: 24px;
    }
    .scrollbar-h .thumb {
      height: 100%;
      min-width: 24px;
    }

    @media (forced-colors: active) {
      .thumb { background: CanvasText; }
    }
  `]

  static properties = {
    orientation: { type: String, reflect: true },
    size: { type: String, reflect: true },
  }

  orientation: 'vertical' | 'horizontal' | 'both' = 'vertical'
  size: 'sm' | 'md' | 'lg' = 'md'

  private vThumbEl: HTMLElement | null = null
  private hThumbEl: HTMLElement | null = null
  private scrollTimer: ReturnType<typeof setTimeout> | null = null

  private get viewportEl(): HTMLElement {
    return this.renderRoot.querySelector('.viewport') as HTMLElement
  }

  private get showV(): boolean {
    return this.orientation === 'vertical' || this.orientation === 'both'
  }
  private get showH(): boolean {
    return this.orientation === 'horizontal' || this.orientation === 'both'
  }

  private updateThumbs = () => {
    const vp = this.viewportEl
    if (!vp) return

    if (this.showV && this.vThumbEl) {
      const ratio = vp.clientHeight / vp.scrollHeight
      if (ratio < 1) {
        const thumbH = Math.max(24, ratio * vp.clientHeight)
        const scrollRatio = vp.scrollTop / (vp.scrollHeight - vp.clientHeight)
        const thumbTop = scrollRatio * (vp.clientHeight - thumbH)
        this.vThumbEl.style.height = `${thumbH}px`
        this.vThumbEl.style.top = `${thumbTop}px`
        this.vThumbEl.style.display = ''
      } else {
        this.vThumbEl.style.display = 'none'
      }
    }

    if (this.showH && this.hThumbEl) {
      const ratio = vp.clientWidth / vp.scrollWidth
      if (ratio < 1) {
        const thumbW = Math.max(24, ratio * vp.clientWidth)
        const scrollRatio = vp.scrollLeft / (vp.scrollWidth - vp.clientWidth)
        const thumbLeft = scrollRatio * (vp.clientWidth - thumbW)
        this.hThumbEl.style.width = `${thumbW}px`
        this.hThumbEl.style.left = `${thumbLeft}px`
        this.hThumbEl.style.display = ''
      } else {
        this.hThumbEl.style.display = 'none'
      }
    }
  }

  private onScroll = () => {
    this.updateThumbs()
    // Show scrollbar while scrolling
    const vBar = this.renderRoot.querySelector('.scrollbar-v') as HTMLElement | null
    const hBar = this.renderRoot.querySelector('.scrollbar-h') as HTMLElement | null
    vBar?.classList.add('active')
    hBar?.classList.add('active')
    if (this.scrollTimer) clearTimeout(this.scrollTimer)
    this.scrollTimer = setTimeout(() => {
      vBar?.classList.remove('active')
      hBar?.classList.remove('active')
    }, 1200)
  }

  firstUpdated() {
    this.vThumbEl = this.renderRoot.querySelector('.scrollbar-v .thumb') as HTMLElement
    this.hThumbEl = this.renderRoot.querySelector('.scrollbar-h .thumb') as HTMLElement
    const vp = this.viewportEl
    if (vp) {
      vp.addEventListener('scroll', this.onScroll, { passive: true })
    }
    this.updateThumbs()
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    if (this.scrollTimer) clearTimeout(this.scrollTimer)
  }

  render() {
    return html`
      <div class="viewport" part="viewport" tabindex="0">
        <slot></slot>
      </div>
      ${this.showV ? html`
        <div class="scrollbar scrollbar-v" part="scrollbar-v" aria-hidden="true">
          <div class="thumb"></div>
        </div>` : ''}
      ${this.showH ? html`
        <div class="scrollbar scrollbar-h" part="scrollbar-h" aria-hidden="true">
          <div class="thumb"></div>
        </div>` : ''}
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-scroll-area')) {
  customElements.define('ship-scroll-area', ShipScrollArea)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-scroll-area': ShipScrollArea
  }
}
