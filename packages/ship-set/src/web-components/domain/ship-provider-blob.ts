/**
 * <ship-provider-blob>
 *
 * Small branded indicator for an AI provider (claude / codex / cursor /
 * gemini / opencode). Inlines the canonical brand SVG from
 * `packages/ship-set/src/assets/provider-logos/` so the blob is always
 * pixel-accurate and offline-safe.
 *
 * Used across the agents surface (list cards, wire-view nodes,
 * create-wizard wiring step) to show which providers an agent compiles to.
 *
 * Attributes:
 *   - provider: claude | codex | cursor | gemini | opencode (required)
 *   - size: sm | md | lg (default md = 20px)
 *   - variant: tile (default) | glyph (bare mark, no chrome)
 *
 * Slots:
 *   - (default) — optional label text rendered next to the tile
 */

import { LitElement, css, html, nothing } from 'lit'
import { unsafeHTML } from 'lit/directives/unsafe-html.js'

// Raw SVG contents inlined at build time via esbuild's text loader.
// These are Simple Icons / vendor-provided marks and carry their
// original brand fill (e.g. `#D97757` for Claude). `currentColor` is
// used where available (Cursor). Themes that want to re-tint a mark
// set `--blob-hue` on the host element; the default keeps the original
// brand colour which is what most callers want.
import claudeSvg from '../../assets/provider-logos/claude.svg'
import cursorSvg from '../../assets/provider-logos/cursor.svg'
import geminiSvg from '../../assets/provider-logos/googlegemini.svg'
import openaiSvg from '../../assets/provider-logos/OpenAI-black-monoblossom.svg'
import openaiDarkSvg from '../../assets/provider-logos/OpenAI-white-monoblossom.svg'
import opencodeSvg from '../../assets/provider-logos/opencode-logo-light.svg'
import opencodeDarkSvg from '../../assets/provider-logos/opencode-logo-dark.svg'

export type Provider = 'claude' | 'codex' | 'cursor' | 'gemini' | 'opencode'
export type BlobSize = 'sm' | 'md' | 'lg'
export type BlobVariant = 'tile' | 'glyph'

interface ProviderMeta {
  label: string
  /** Brand-hinted tint used for the tile's soft background + outline. */
  hue: string
  /** Light-mode SVG content. */
  svg: string
  /** Optional dark-mode variant. If omitted, `svg` is used. */
  svgDark?: string
}

const PROVIDERS: Record<Provider, ProviderMeta> = {
  claude:   { label: 'Claude',   hue: 'oklch(0.70 0.16 35)',  svg: claudeSvg as unknown as string },
  codex:    { label: 'Codex',    hue: 'oklch(0.22 0.02 220)', svg: openaiSvg as unknown as string, svgDark: openaiDarkSvg as unknown as string },
  cursor:   { label: 'Cursor',   hue: 'oklch(0.45 0.02 250)', svg: cursorSvg as unknown as string },
  gemini:   { label: 'Gemini',   hue: 'oklch(0.62 0.14 285)', svg: geminiSvg as unknown as string },
  opencode: { label: 'OpenCode', hue: 'oklch(0.60 0.18 35)',  svg: opencodeSvg as unknown as string, svgDark: opencodeDarkSvg as unknown as string },
}

export class ShipProviderBlob extends LitElement {
  static styles = css`
    :host {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      vertical-align: middle;
      font-family: var(--font-body);
      font-size: var(--text-xs);
      color: var(--muted-foreground);
      --blob-size: 20px;
    }
    :host([size='sm']) { --blob-size: 14px; font-size: 11px; }
    :host([size='md']) { --blob-size: 20px; }
    :host([size='lg']) { --blob-size: 28px; font-size: var(--text-sm); }
    .tile {
      width: var(--blob-size);
      height: var(--blob-size);
      flex-shrink: 0;
      border-radius: calc(var(--blob-size) * 0.22);
      display: grid; place-items: center;
      background: color-mix(in oklch, var(--blob-hue, var(--muted)) 14%, var(--surface-1));
      box-shadow: inset 0 0 0 1px color-mix(in oklch, var(--blob-hue, var(--foreground)) 20%, transparent);
      overflow: hidden;
    }
    :host([variant='glyph']) .tile {
      background: transparent;
      box-shadow: none;
    }
    .tile svg {
      width: 72%;
      height: 72%;
      display: block;
    }
    :host([variant='glyph']) .tile svg { width: 100%; height: 100%; }
    .label { white-space: nowrap; text-transform: capitalize; }
    .label:empty { display: none; }
  `

  static properties = {
    provider: { type: String, reflect: true },
    size: { type: String, reflect: true },
    variant: { type: String, reflect: true },
  }

  provider: Provider = 'claude'
  size: BlobSize = 'md'
  variant: BlobVariant = 'tile'

  render() {
    const meta = PROVIDERS[this.provider] ?? PROVIDERS.claude
    this.style.setProperty('--blob-hue', meta.hue)
    // Prefer light SVG; callers can opt into dark via CSS class if needed.
    const svg = meta.svg
    return html`
      <span class="tile" role="img" aria-label=${meta.label}>${unsafeHTML(svg)}</span>
      <span class="label"><slot>${nothing}</slot></span>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-provider-blob')) {
  customElements.define('ship-provider-blob', ShipProviderBlob)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-provider-blob': ShipProviderBlob
  }
}
