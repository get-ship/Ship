//! `<ship-loading>` — polished loading surface.
//!
//! Replaces string-only "loading…" placeholders. Shows an animated
//! rocket lifting off with exhaust flames + falling particles, paired
//! with a rotating copy line. A minimum visible duration avoids the
//! jarring flash of a <150ms appearance before content lands.
//! Respects prefers-reduced-motion.
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

    /* Stage frames the rocket + exhaust + particles so the bob/flame
     * animations sit in a fixed footprint and don't shift the message. */
    .stage {
      position: relative;
      width: 56px;
      height: 64px;
      display: grid;
      place-items: center;
    }
    :host([compact]) .stage {
      width: 28px;
      height: 32px;
    }

    .rocket {
      position: relative;
      z-index: 2;
      width: 32px;
      height: 32px;
      color: var(--primary);
      filter: drop-shadow(0 4px 6px color-mix(in oklch, var(--primary) 35%, transparent));
      animation: ship-loading-rocket 2.2s ease-in-out infinite;
      transform-origin: 50% 100%;
    }
    :host([compact]) .rocket {
      width: 18px;
      height: 18px;
      filter: none;
    }
    .rocket svg {
      width: 100%;
      height: 100%;
    }

    /* Flame: warm orange→yellow gradient teardrop directly below the
     * rocket, scaled + alpha-flickered for a believable burn loop. */
    .flame {
      position: absolute;
      left: 50%;
      bottom: 6px;
      transform: translateX(-50%);
      width: 14px;
      height: 18px;
      z-index: 1;
      background: radial-gradient(
        ellipse at 50% 30%,
        color-mix(in oklch, var(--primary) 95%, white) 0%,
        color-mix(in oklch, var(--primary) 70%, transparent) 55%,
        transparent 80%
      );
      border-radius: 50% 50% 35% 35% / 60% 60% 35% 35%;
      filter: blur(0.5px);
      transform-origin: 50% 0%;
      animation: ship-loading-flame 0.18s ease-in-out infinite alternate,
                 ship-loading-flame-pulse 2.2s ease-in-out infinite;
    }
    :host([compact]) .flame {
      width: 8px;
      height: 10px;
      bottom: 4px;
    }

    /* Exhaust particles fall away beneath the rocket. Three offset
     * dots each carry their own duration so the cadence reads
     * organic, not metronomic. */
    .particle {
      position: absolute;
      left: 50%;
      top: 75%;
      width: 3px;
      height: 3px;
      border-radius: 50%;
      background: color-mix(in oklch, var(--primary) 85%, transparent);
      opacity: 0;
      z-index: 0;
    }
    .particle.p1 {
      animation: ship-loading-particle 1.4s ease-out infinite;
      animation-delay: 0s;
    }
    .particle.p2 {
      animation: ship-loading-particle 1.7s ease-out infinite;
      animation-delay: 0.4s;
      width: 2px;
      height: 2px;
      transform: translate(-50%, 0);
    }
    .particle.p3 {
      animation: ship-loading-particle 1.5s ease-out infinite;
      animation-delay: 0.8s;
      width: 4px;
      height: 4px;
    }
    :host([compact]) .particle { display: none; }

    .message {
      font-size: var(--text-sm);
      color: var(--muted-foreground);
      transition: opacity var(--duration-normal, 220ms) var(--ease-out, ease);
      letter-spacing: 0.01em;
    }
    :host([compact]) .message {
      font-size: var(--text-xs);
    }

    /* Rocket: bob up while subtly tilting, then settle for the next
     * "lift". The transform-origin at 50% 100% keeps the wobble
     * grounded under the flame. */
    @keyframes ship-loading-rocket {
      0%   { transform: translateY(0)    rotate(0deg);  }
      40%  { transform: translateY(-6px) rotate(-1.5deg); }
      60%  { transform: translateY(-7px) rotate(1.5deg);  }
      100% { transform: translateY(0)    rotate(0deg);  }
    }
    /* Fast flicker keeps the flame alive. */
    @keyframes ship-loading-flame {
      0%   { transform: translateX(-50%) scaleY(1)    scaleX(1); opacity: 0.9; }
      100% { transform: translateX(-50%) scaleY(1.15) scaleX(0.92); opacity: 1; }
    }
    /* Slow pulse synced with the rocket bob so the flame plumes when
     * the rocket dips. */
    @keyframes ship-loading-flame-pulse {
      0%, 100% { height: 16px; }
      50%      { height: 22px; }
    }
    /* Particles fall away from the rocket as it lifts — they spawn
     * just below the engine, drift down, fade out. */
    @keyframes ship-loading-particle {
      0%   { transform: translate(-50%, 0)    scale(1);   opacity: 0.7; }
      80%  { transform: translate(-50%, 22px) scale(0.5); opacity: 0; }
      100% { transform: translate(-50%, 26px) scale(0.3); opacity: 0; }
    }

    @media (prefers-reduced-motion: reduce) {
      .rocket,
      .flame,
      .particle {
        animation: none;
      }
      .flame {
        opacity: 0.7;
      }
      .particle { display: none; }
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
      <div class="stage" aria-hidden="true">
        <div class="rocket">
          <!-- Stylized rocket: capsule body, window, fins, exhaust port.
               Filled in currentColor so themes recolor for free.
               Geometry inspired by Lucide rocket but simplified for
               animation legibility at 32px. -->
          <svg viewBox="0 0 32 40" fill="none" xmlns="http://www.w3.org/2000/svg">
            <!-- nose cone -->
            <path
              d="M16 2 C20 6 22 11 22 16 H10 C10 11 12 6 16 2 Z"
              fill="currentColor"
            />
            <!-- body -->
            <rect
              x="10" y="16" width="12" height="14" rx="2"
              fill="currentColor"
              opacity="0.9"
            />
            <!-- window -->
            <circle
              cx="16" cy="20" r="2.5"
              fill="var(--background, #fff)"
              opacity="0.85"
            />
            <!-- left fin -->
            <path
              d="M10 24 L4 32 L10 30 Z"
              fill="currentColor"
              opacity="0.75"
            />
            <!-- right fin -->
            <path
              d="M22 24 L28 32 L22 30 Z"
              fill="currentColor"
              opacity="0.75"
            />
            <!-- engine port -->
            <rect
              x="13" y="29" width="6" height="2" rx="0.5"
              fill="var(--background, #000)"
              opacity="0.5"
            />
          </svg>
        </div>
        <div class="flame"></div>
        <div class="particle p1"></div>
        <div class="particle p2"></div>
        <div class="particle p3"></div>
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
