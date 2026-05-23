import type { SuiBuildConfig } from '@ship/sui-build'

// `apps/shell` — Ship chrome `<sui-*>` components. Live as shell-local
// elements rather than a published pack: the walker tags them
// `app:shell` so they appear in `workspace.json` but never in
// `published.json`.
//
// The `sui-chrome` aggregate registers every non-separate chrome tag
// (view-picker, theme-picker, status-bar, brand-mark, view-chrome,
// edit-overlay, edit-mode-pill, annotation-*); it lands under
// `.deck/sui-components/sui-chrome.js` so the daemon's existing
// `serve_sui_island` handler serves it at
// `/_sui/sui-components/sui-chrome.js` — no new route needed. The
// `index.html` IIFE loads it eagerly alongside the umbrella aggregate.
//
// Chrome is the shell *app*'s content, so it builds to the project's `.deck/`
// (deck's build output) — never into deck's own crate. deck-server reads it
// from the host-supplied `.deck` root.
//
// Heavy chrome (terminal, code-editor, markdown-editor, command-bar,
// launcher, nav-top, panel-dock, rail, devtools, component-picker,
// view-authoring) opts into its own code-split bundle via
// `<bundle separate />` in its `.sui` source; outputs land alongside
// the aggregate under `assets/sui-components/`.

const config: SuiBuildConfig = {
  entries: {
    'sui-components/sui-chrome': 'src/components/index.ts',
  },
  lazyBundles: {
    'sui-agent-attach':     'src/components/sui-agent-attach/index.ts',
    'sui-terminal':         'src/components/sui-terminal/index.ts',
    'sui-code-editor':      'src/components/sui-code-editor/index.ts',
    'sui-markdown-editor':  'src/components/sui-markdown-editor/index.ts',
    'sui-command-bar':      'src/components/sui-command-bar/index.ts',
    'sui-launcher':         'src/components/sui-launcher/index.ts',
    'sui-nav-top':          'src/components/sui-nav-top/index.ts',
    'sui-panel-dock':       'src/components/sui-panel-dock/index.ts',
    'sui-rail':             'src/components/sui-rail/index.ts',
    'sui-devtools':         'src/components/sui-devtools/index.ts',
    'sui-component-picker': 'src/components/sui-component-picker/index.ts',
    'sui-view-authoring':   'src/components/sui-view-authoring/index.ts',
    'sui-account-menu':     'src/components/sui-account-menu/index.ts',
  },
  // Studio VIEW trees (`src/studio/**/page.sui` + nested chrome elements).
  // These are loaded by the daemon view-host, not bundled into the chrome
  // aggregate, but their controllers import the generated `Scope` from a
  // sibling `.sui`. Compiling them emits each `page.sui.d.ts` into the
  // mirror tree so those `./page.sui` type imports resolve under tsc's
  // `rootDirs`. Recursive — covers dynamic `[id]`/`[name]` route segments.
  compileDirs: ['src/studio'],
  outdir: '../../.deck',
  lazyOutSubdir: 'sui-components',
}

export default config
