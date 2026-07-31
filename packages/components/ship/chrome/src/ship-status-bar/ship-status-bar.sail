<controller>
// Status-bar controller — wires the chrome bottom strip.
// Migrated from apps/shell/src/components/sui-status-bar; the launch-chrome
// surface (branch/theme/online/agents/mode/project props, status slot, click
// intents) is the source of truth.
//
// Responsibilities:
//   1. Click handlers for the project, daemon-dot, and theme buttons —
//      each dispatches a composed CustomEvent the shell listens for.
//   2. A view-extensible status zone driven by `chrome.status.set(...)`
//      (the runtime emits sui:status:set / sui:status:clear). Entries
//      survive across re-mounts of the host because the controller
//      subscribes at the document level and re-renders from the live map.
//   3. Sets host a11y attributes (role=contentinfo) so screen-reader
//      navigation announces the bar.

import { controller, signal } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-status-bar.sail'

interface StatusValue {
  label: string
  icon?: string
  tone?: 'default' | 'info' | 'success' | 'warning' | 'error'
  tooltip?: string
}

interface ExtensionEntry extends StatusValue {
  key: string
}

interface SetDetail { key?: string; value?: StatusValue }
interface ClearDetail { key?: string }

function dispatch(host: HTMLElement, type: string, detail?: unknown): void {
  host.dispatchEvent(new CustomEvent(type, {
    detail: detail ?? null,
    bubbles: true,
    composed: true,
  }))
}

export default controller((_ctx: ViewScope<Scope>, host?: Element) => {

    const extensions = signal<ExtensionEntry[]>([])
    const hostEl = host as HTMLElement

    function onDotClick(): void {
      dispatch(hostEl, 'sui:dock:toggle')
    }
    function onProjectClick(): void {
      dispatch(hostEl, 'sui:project-switcher:open')
    }
    function onThemeClick(): void {
      dispatch(hostEl, 'sui:theme-picker:open')
    }

    return { extensions, onDotClick, onProjectClick, onThemeClick }
})
</controller>

<!--
  <ship-status-bar> — @ship/chrome · the bottom strip. Glance-only system status.
  Lives in the chrome pack (migrated from apps/shell/src/components/sui-status-bar).
  Hosts:
    daemon-dot · project · branch · agents · view-extensions ··· end-slot · dev · theme

  Click behavior (intent):
    project       → opens project switcher (sui:project-switcher:open)
    daemon dot    → opens devtools dock (sui:dock:toggle)
    theme         → opens theme picker (sui:theme-picker:open)

  Live-updating via `chrome.status.set('key', { label, icon, tone, tooltip })`.
  The controller subscribes to sui:status:set / sui:status:clear and renders
  entries into an aria-live="polite" region.

  Prop surface
  ─────────────────────────────────────────────────────────────────────────────
  branch   (string, '')              git branch text
  theme    (string, '')              current theme ID
  online   (bool, false, reflect)    daemon connection state — dot tone
  agents   (string, '')              local agent indicator text
  mode     (string, '', reflect)     'development' renders DEV badge
  project  (string, '')              active project label (shown when set)
  ─────────────────────────────────────────────────────────────────────────────
  Slots:
    start  — view-injected status content between branch and extensions
    end    — view-injected status content between spacer and theme
  Parts: project, dot-btn, dot, branch, agents, extensions, mode, theme-btn
  ─────────────────────────────────────────────────────────────────────────────
-->

<!-- <prop name="branch" type="string" default="" /> -->
<!-- <prop name="theme" type="string" default="" /> -->
<!-- <prop name="online" type="bool" default="false" reflect /> -->
<!-- <prop name="agents" type="string" default="" /> -->
<!-- <prop name="mode" type="string" default="" reflect /> -->
<!-- <prop name="project" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<button type="button"
  on:click={onDotClick}
  title="Daemon — click to toggle dev panel"
  aria-label="Daemon connection status. Click to toggle dev panel.">
  <span aria-hidden="true"></span>
</button>
<if cond={project()}>
  <button type="button"
    on:click={onProjectClick}
    title="Active project — click to switch"
    aria-label="Active project. Click to switch.">
    <span aria-hidden="true"></span>
    <span>{project()}</span>
  </button>
</if>
<span>{branch()}</span>
<span>{agents()}</span>
<span><slot name="start"></slot></span>
<span role="status" aria-live="polite" aria-label="View status">
  <for as="ext" of={extensions()} key={ext.key}>
    <span data-tone={ext.tone} title={ext.tooltip}>
      <span>{ext.label}</span>
    </span>
  </for>
</span>
<span></span>
<span><slot name="end"></slot></span>
<span aria-label="Daemon mode">dev</span>
<button type="button"
  on:click={onThemeClick}
  title="Active theme — click to open theme picker"
  aria-label="Active theme. Click to open theme picker.">{theme()}</button>
