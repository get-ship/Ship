// Heavy chrome — code-split dynamic imports.
//
// Each import() below creates a separate Rolldown chunk so the browser can
// fetch these heavier surfaces in parallel without blocking first paint.
// Custom elements upgrade in place when their chunk lands, matching the
// previous async island behaviour (/_sui/sui-components/<tag>.js) that is
// now deleted.
//
// Call loadHeavyChrome() early in boot — fire-and-forget, do NOT await.
// The shell mounts its chrome surfaces (sui-nav-top, sui-launcher, …) before
// this resolves; they upgrade lazily once their registration chunk arrives.
//
// Each entry point is the component's TypeScript index which itself uses the
// `<tag>.compiled.js` convention — the SuiPlugin compiles the sibling `.sui`
// in-process so no pre-built artefacts are needed.

export function loadHeavyChrome(): void {
  // Top navigation bar — breadcrumbs, command-bar slot, account, dock toggle.
  void import('../components/sui-nav-top/index')
  // Command bar + sub-surfaces (palette, chat). The index bundles all three
  // because command-chat and command-palette are authored inside the same dir.
  void import('../components/sui-command-bar/index')
  // App launcher — floating panel opened via sui:launcher:open.
  void import('../components/sui-launcher/index')
  // Bottom panel dock — terminal + devtools tab bar.
  void import('../components/sui-panel-dock/index')
  // Account menu — user identity / sign-out popover.
  void import('../components/sui-account-menu/index')
  // Project switcher — workspace selector popover.
  void import('../components/sui-project-switcher/index')
  // Devtools panels — four slots inside <sui-panel-dock>:
  //   <sui-devtools-source slot="source">
  //   <sui-devtools-signals slot="signals">
  //   <sui-devtools-schema slot="schema">
  //   <sui-devtools-components slot="components">
  // Bundled in one chunk (loaded together, always used together).
  void import('../components/sui-devtools/index')
}
