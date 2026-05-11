//! `<ship-edit-overlay>` — hover outline + click-to-annotate overlay
//! for edit mode.
//!
//! Activation: ⌘/Ctrl+⇧+E toggles `body.edit-mode`. This element
//! watches that class and mounts/unmounts pointer listeners on
//! `#ship-view-root`. Hover paints an outline box + tag label
//! absolutely positioned over the hovered element. Click stages an
//! annotation by dispatching `ship:chat:annotate` on `<ship-shell>`
//! with a minimal identifier payload; the chat drawer already
//! subscribes and renders a chip.
//!
//! The overlay lives in light DOM so the absolute-positioned
//! highlight uses the host page's stacking context, and the visual
//! affordance stays on top of the mounted view without interfering
//! with it (pointer-events: none on the box itself).

import { LitElement, css, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";

interface Highlight {
  top: number;
  left: number;
  width: number;
  height: number;
  tag: string;
  id: string | null;
}

function targetFromEvent(e: PointerEvent | MouseEvent): HTMLElement | null {
  const root = document.getElementById("ship-view-root");
  if (!root) return null;
  const target = e.target as HTMLElement | null;
  if (!target || !root.contains(target)) return null;
  // Ignore the overlay itself if the user somehow hits it.
  if (target.closest("ship-edit-overlay")) return null;
  return target;
}

function describe(el: HTMLElement): { tag: string; selector: string; id: string | null } {
  const tag = el.tagName.toLowerCase();
  const id = el.id || null;
  let selector = tag;
  if (id) selector = `#${id}`;
  else if (el.classList.length > 0) {
    selector = `${tag}.${Array.from(el.classList).slice(0, 2).join(".")}`;
  }
  return { tag, selector, id };
}

@customElement("ship-edit-overlay")
export class ShipEditOverlay extends LitElement {
  static styles = css`
    :host {
      position: fixed;
      inset: 0;
      pointer-events: none;
      z-index: 9500;
      display: none;
    }
    :host(.active) {
      display: block;
    }
    .box {
      position: absolute;
      border: 2px solid var(--primary);
      background: color-mix(in oklch, var(--primary) 8%, transparent);
      border-radius: 2px;
      box-shadow: 0 0 0 1px color-mix(in oklch, var(--primary) 40%, transparent);
      transition: all 60ms linear;
    }
    .label {
      position: absolute;
      padding: 2px 6px;
      font-family: var(--font-mono);
      font-size: 11px;
      color: var(--primary-foreground, #fff);
      background: var(--primary);
      border-radius: 2px;
      white-space: nowrap;
      transform: translateY(-100%);
    }
    .hint {
      position: fixed;
      bottom: 40px;
      left: 50%;
      transform: translateX(-50%);
      padding: 6px 12px;
      background: var(--card, var(--background));
      border: 1px solid var(--border);
      border-radius: var(--radius-md, 6px);
      color: var(--muted-foreground);
      font-size: var(--text-xs);
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
    }
    .hint kbd {
      font-family: var(--font-mono);
      background: var(--muted);
      padding: 1px 4px;
      border-radius: 3px;
      border: 1px solid var(--border);
      margin: 0 2px;
    }
  `;

  @state() private _active = document.body.classList.contains("edit-mode");
  @state() private _hl: Highlight | null = null;

  private _observer: MutationObserver | null = null;
  private _onPointerMove = (e: PointerEvent): void => {
    const target = targetFromEvent(e);
    if (!target) {
      this._hl = null;
      return;
    }
    const rect = target.getBoundingClientRect();
    const { tag, id } = describe(target);
    this._hl = {
      top: rect.top,
      left: rect.left,
      width: rect.width,
      height: rect.height,
      tag,
      id,
    };
  };
  private _onPointerLeave = (): void => {
    this._hl = null;
  };
  private _onClick = (e: MouseEvent): void => {
    const target = targetFromEvent(e);
    if (!target) return;
    // In edit mode, swallow the click so the underlying view doesn't
    // navigate / toggle / whatever the normal click would do.
    e.preventDefault();
    e.stopPropagation();
    const { tag, selector, id } = describe(target);
    this.closest("ship-shell")?.dispatchEvent(
      new CustomEvent("ship:chat:annotate", {
        detail: {
          tag,
          selector,
          id,
          // Short preview of text content to jog the user's memory.
          preview: (target.textContent ?? "").trim().slice(0, 60),
        },
        bubbles: true,
      }),
    );
    // Ensure chat is visible so the user sees the chip land.
    window.ship?.chat?.open?.();
  };

  createRenderRoot(): HTMLElement | DocumentFragment {
    return this;
  }

  connectedCallback(): void {
    super.connectedCallback();
    this._applyActive();
    this._observer = new MutationObserver(() => this._applyActive());
    this._observer.observe(document.body, {
      attributes: true,
      attributeFilter: ["class"],
    });
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this._observer?.disconnect();
    this._observer = null;
    this._teardownListeners();
  }

  private _applyActive(): void {
    const on = document.body.classList.contains("edit-mode");
    if (on === this._active) return;
    this._active = on;
    this.classList.toggle("active", on);
    if (on) this._setupListeners();
    else {
      this._teardownListeners();
      this._hl = null;
    }
  }

  private _setupListeners(): void {
    const root = document.getElementById("ship-view-root");
    if (!root) return;
    root.addEventListener("pointermove", this._onPointerMove, true);
    root.addEventListener("pointerleave", this._onPointerLeave, true);
    root.addEventListener("click", this._onClick, true);
  }

  private _teardownListeners(): void {
    const root = document.getElementById("ship-view-root");
    if (!root) return;
    root.removeEventListener("pointermove", this._onPointerMove, true);
    root.removeEventListener("pointerleave", this._onPointerLeave, true);
    root.removeEventListener("click", this._onClick, true);
  }

  render() {
    if (!this._active) return nothing;
    return html`
      ${this._hl
        ? html`
            <div
              class="box"
              style=${`top:${this._hl.top}px;left:${this._hl.left}px;width:${this._hl.width}px;height:${this._hl.height}px`}
            ></div>
            <div
              class="label"
              style=${`top:${this._hl.top}px;left:${this._hl.left}px`}
            >
              &lt;${this._hl.tag}${this._hl.id ? `#${this._hl.id}` : ""}&gt;
            </div>
          `
        : nothing}
      <div class="hint">
        Edit mode — hover an element, click to annotate.
        <kbd>⌘⇧E</kbd> to exit.
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-edit-overlay": ShipEditOverlay;
  }
}
