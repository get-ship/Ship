/**
 * <ship-app-icon>
 *
 * Identity glyph for a Ship app surface — used by the rail, dock,
 * launcher, breadcrumbs, and anywhere a single app needs to be
 * named visually. Every Ship app has one canonical Lucide glyph
 * and one canonical hue token; this component is the single source
 * of truth for that mapping.
 *
 * Attributes:
 *   - name: app id. One of:
 *       dashboard | pipelines | agents | skills | git | collections |
 *       canvas | settings | chat | events | specs | theme | setup | overview
 *     Unknown names render the slotted fallback or a neutral "dot" glyph.
 *   - variant: glyph (default, no tile) | tile (soft tinted bg) | solid (filled accent bg)
 *   - size: sm | md | lg (default md). Maps to 16/20/28px tile, 12/14/18 glyph.
 *   - radius: sm | md | lg | full — overrides the size-derived default.
 *   - label: when set, host gets role=img + aria-label; otherwise aria-hidden.
 *
 * Slots:
 *   - (default) — escape hatch for marketplace / extension views to
 *     provide a custom glyph (raw <svg>, an <img>, or a <ship-icon>).
 *
 * The hue is read off the --app-accent-{name} channel from
 * tokens.css. Themes override that channel; this component reads it
 * via the per-host `--app-accent` variable, same pattern as
 * `<ship-button>`.
 */

import { LitElement, css, html, nothing } from 'lit'
// Side-effect import — guarantees `<ship-icon>` is registered whenever
// `<ship-app-icon>` is loaded, so the inner glyph renders even when the
// component is mounted in an isolated context (e.g. `page.setContent`)
// without an explicit registry-init call from the host app.
import './ship-icon'

type AppName =
  | 'dashboard' | 'pipelines' | 'agents' | 'skills' | 'git'
  | 'collections' | 'canvas' | 'settings' | 'chat' | 'events'
  | 'specs' | 'theme' | 'setup' | 'overview'

type AppVariant = 'glyph' | 'tile' | 'solid'
type AppSize = 'sm' | 'md' | 'lg'

const APP_GLYPH: Record<AppName, string> = {
  dashboard: 'layout-dashboard',
  pipelines: 'workflow',
  agents: 'bot',
  skills: 'sparkles',
  git: 'git-branch',
  collections: 'database',
  canvas: 'frame',
  settings: 'settings',
  chat: 'messages-square',
  events: 'radio',
  specs: 'file-text',
  theme: 'palette',
  setup: 'rocket',
  overview: 'compass',
}

const APP_NAMES = new Set(Object.keys(APP_GLYPH))

export class ShipAppIcon extends LitElement {
  static styles = css`
    :host {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      vertical-align: middle;

      /* Per-host channels. Subclassed below by :host([name=…]). */
      --app-accent: var(--foreground);

      /* Size channel — tile dimensions and glyph size scale together
       * and themes can rescale by overriding the channel. */
      --tile-size: 20px;
      --glyph-size: 14px;
      --tile-radius: var(--radius-md);

      width: var(--tile-size);
      height: var(--tile-size);
      color: var(--app-accent);
    }
    :host([hidden]) { display: none; }

    /* Per-app accent channel. */
    :host([name='dashboard'])   { --app-accent: var(--app-accent-dashboard); }
    :host([name='pipelines'])   { --app-accent: var(--app-accent-pipelines); }
    :host([name='agents'])      { --app-accent: var(--app-accent-agents); }
    :host([name='skills'])      { --app-accent: var(--app-accent-skills); }
    :host([name='git'])         { --app-accent: var(--app-accent-git); }
    :host([name='collections']) { --app-accent: var(--app-accent-collections); }
    :host([name='canvas'])      { --app-accent: var(--app-accent-canvas); }
    :host([name='settings'])    { --app-accent: var(--app-accent-settings); }
    :host([name='chat'])        { --app-accent: var(--app-accent-chat); }
    :host([name='events'])      { --app-accent: var(--app-accent-events); }
    :host([name='specs'])       { --app-accent: var(--app-accent-specs); }
    :host([name='theme'])       { --app-accent: var(--app-accent-theme); }
    :host([name='setup'])       { --app-accent: var(--app-accent-setup); }
    :host([name='overview'])    { --app-accent: var(--app-accent-overview); }

    /* Size channel. */
    :host([size='sm']) { --tile-size: 16px; --glyph-size: 12px; --tile-radius: var(--radius-sm); }
    :host([size='md']) { --tile-size: 20px; --glyph-size: 14px; --tile-radius: var(--radius-md); }
    :host([size='lg']) { --tile-size: 28px; --glyph-size: 18px; --tile-radius: var(--radius-lg); }

    /* Radius override channel. */
    :host([radius='sm'])   { --tile-radius: var(--radius-sm); }
    :host([radius='md'])   { --tile-radius: var(--radius-md); }
    :host([radius='lg'])   { --tile-radius: var(--radius-lg); }
    :host([radius='full']) { --tile-radius: var(--radius-full); }

    .frame {
      width: 100%;
      height: 100%;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      border-radius: var(--tile-radius);
      transition: background-color var(--duration-fast) var(--ease-out),
                  color var(--duration-fast) var(--ease-out),
                  box-shadow var(--duration-fast) var(--ease-out);
    }

    /* glyph (default) — bare icon, no tile. */
    :host(:not([variant])) .frame,
    :host([variant='glyph']) .frame {
      background: transparent;
      color: var(--app-accent);
    }

    /* tile — soft tinted bg, accent fg. */
    :host([variant='tile']) .frame {
      background: color-mix(in oklab, var(--app-accent) 16%, transparent);
      color: var(--app-accent);
    }

    /* solid — filled accent bg, contrast fg. The contrast color reads
     * --background so the glyph stays legible across themes. */
    :host([variant='solid']) .frame {
      background: var(--app-accent);
      color: var(--background);
      box-shadow: var(--shadow-xs);
    }

    /* Glyph size — both inline svg and slotted content scale to channel. */
    ship-icon {
      width: var(--glyph-size);
      height: var(--glyph-size);
    }
    ::slotted(*) {
      width: var(--glyph-size);
      height: var(--glyph-size);
      display: block;
    }
  `

  static properties = {
    name: { type: String, reflect: true },
    variant: { type: String, reflect: true },
    size: { type: String, reflect: true },
    radius: { type: String, reflect: true },
    label: { type: String, reflect: true },
  }

  name: AppName | string = ''
  variant: AppVariant = 'glyph'
  size: AppSize = 'md'
  radius: '' | 'sm' | 'md' | 'lg' | 'full' = ''
  label = ''

  updated() {
    if (this.label) {
      this.setAttribute('role', 'img')
      this.setAttribute('aria-label', this.label)
      this.removeAttribute('aria-hidden')
    } else {
      this.setAttribute('aria-hidden', 'true')
      this.removeAttribute('role')
      this.removeAttribute('aria-label')
    }
  }

  render() {
    const known = APP_NAMES.has(this.name)
    const glyph = known ? APP_GLYPH[this.name as AppName] : 'dot'
    return html`
      <span class="frame" part="frame">
        <slot>
          <ship-icon name=${glyph} part="glyph"
            aria-hidden=${this.label ? nothing : 'true'}></ship-icon>
        </slot>
      </span>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-app-icon')) {
  customElements.define('ship-app-icon', ShipAppIcon)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-app-icon': ShipAppIcon
  }
}
