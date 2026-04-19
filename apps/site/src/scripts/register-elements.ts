// Narrow registration of the ship web components used on the marketing site.
// Collapses the dynamic imports into a single chunk (via inlineDynamicImports)
// and defers upgrading until the browser is idle.

const load = async () => {
  await Promise.all([
    import('@ship/ui/web-components/ship-button'),
    import('@ship/ui/web-components/ship-card'),
    import('@ship/ui/web-components/ship-badge'),
    import('@ship/ui/web-components/ship-select'),
    import('@ship/ui/web-components/ship-separator'),
    import('@ship/ui/web-components/ship-status-dot'),
  ]);
};

const ric = (cb: () => void) => {
  const anyWin = window as any;
  if (typeof anyWin.requestIdleCallback === 'function') {
    anyWin.requestIdleCallback(cb, { timeout: 4000 });
  } else {
    setTimeout(cb, 500);
  }
};

const bootstrap = () => ric(() => void load());

if (document.readyState === 'complete') {
  bootstrap();
} else {
  window.addEventListener('load', bootstrap, { once: true });
}
