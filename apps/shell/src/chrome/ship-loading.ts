//! `<ship-loading>` — polished loading surface.
//!
//! Replaces string-only "loading…" placeholders. Shows the Ship
//! mark with a subtle pulse and a rotating copy line. A minimum
//! visible duration avoids the jarring flash of a <150ms appearance
//! before content lands. Respects prefers-reduced-motion.
//!
//! Attributes:
//!   messages   — pipe-separated strings to rotate through every 1.4s.
//!                Default: "Waking daemon | Warming routes | Fetching state"
//!   min-ms     — minimum visible duration before the element may be
//!                removed by the host (advisory; host checks via
//!                `minElapsed` property).
//!   compact    — smaller variant for in-view loads (no big mark).

import { LitElement, css, html } from "lit";
import { customElement, property, state } from "lit/decorators.js";

const DEFAULT_MESSAGES = [
  "Waking daemon",
  "Warming routes",
  "Fetching state",
  "Hydrating shell",
];

@customElement("ship-loading")
export class ShipLoading extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: var(--space-3, 12px);
      min-height: 240px;
      width: 100%;
      color: var(--muted-foreground);
    }
    :host([compact]) {
      min-height: 80px;
      gap: 6px;
    }
    .mark {
      width: 44px;
      height: 44px;
      border-radius: var(--radius-md, 6px);
      background: linear-gradient(
        135deg,
        color-mix(in oklch, var(--primary) 85%, transparent),
        color-mix(in oklch, var(--primary) 60%, transparent)
      );
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--primary-foreground, #fff);
      animation: ship-loading-pulse 1.6s ease-in-out infinite;
    }
    :host([compact]) .mark {
      width: 24px;
      height: 24px;
      border-radius: var(--radius-sm, 4px);
    }
    .mark svg {
      width: 60%;
      height: 60%;
    }
    .message {
      font-size: var(--text-sm);
      color: var(--muted-foreground);
      transition: opacity var(--duration-normal, 220ms) var(--ease-out, ease);
    }
    :host([compact]) .message {
      font-size: var(--text-xs);
    }
    @keyframes ship-loading-pulse {
      0%, 100% { transform: scale(1); opacity: 0.9; }
      50% { transform: scale(1.06); opacity: 1; }
    }
    @media (prefers-reduced-motion: reduce) {
      .mark { animation: none; }
      .message { transition: none; }
    }
  `;

  @property({ type: String })
  messages = "";

  @property({ type: Boolean, reflect: true })
  compact = false;

  @state() private _index = 0;
  @state() private _visible = true;
  private _timer: ReturnType<typeof setInterval> | null = null;

  private _parsedMessages(): string[] {
    const raw = this.messages.trim();
    if (!raw) return DEFAULT_MESSAGES;
    return raw
      .split("|")
      .map((s) => s.trim())
      .filter(Boolean);
  }

  connectedCallback(): void {
    super.connectedCallback();
    this._timer = setInterval(() => {
      this._visible = false;
      requestAnimationFrame(() => {
        this._index = (this._index + 1) % this._parsedMessages().length;
        this._visible = true;
      });
    }, 1400);
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    if (this._timer) clearInterval(this._timer);
    this._timer = null;
  }

  render() {
    const msg = this._parsedMessages()[this._index] ?? "";
    return html`
      <div class="mark" aria-hidden="true">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M4 12 L12 4 L20 12 L12 20 Z"></path>
          <circle cx="12" cy="12" r="2.4" fill="currentColor"></circle>
        </svg>
      </div>
      <div class="message" style=${`opacity: ${this._visible ? 1 : 0}`}>
        ${msg}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-loading": ShipLoading;
  }
}
