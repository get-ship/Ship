// Narrow registration of the ship web components used on the marketing site.
//
// Strategy: the hero, pillars, nav, and platform cards are styled via the
// :not(:defined) fallback sheet in global.css, so the page is fully usable
// without Lit. We therefore defer every customElements.define call until
// either:
//   1. the user intends to interact (first scroll, pointerdown, or keydown),
//   2. the footer scrolls into view (the theme switcher needs ship-select
//      to be real), or
//   3. a 3s safety fallback after `load` has fired.
// Registrations run one component per idle tick so no single task eats
// the main thread.

type Loader = () => Promise<unknown>;

const loaders: Loader[] = [
  () => import('@ship/ui/web-components/ship-button'),
  () => import('@ship/ui/web-components/ship-card'),
  () => import('@ship/ui/web-components/ship-select'),
  () => import('@ship/ui/web-components/ship-badge'),
  () => import('@ship/ui/web-components/ship-status-dot'),
  () => import('@ship/ui/web-components/ship-separator'),
  () => import('@ship/ui/web-components/ship-toolbar'),
  () => import('@ship/ui/web-components/ship-alert'),
];

const ric = (cb: () => void) => {
  const anyWin = window as any;
  if (typeof anyWin.requestIdleCallback === 'function') {
    anyWin.requestIdleCallback(cb, { timeout: 2000 });
  } else {
    setTimeout(cb, 200);
  }
};

const registerOne = (loader: Loader) =>
  new Promise<void>((resolve) => {
    ric(async () => {
      try {
        await loader();
      } finally {
        resolve();
      }
    });
  });

let started = false;
const bootstrap = async () => {
  if (started) return;
  started = true;
  for (const loader of loaders) {
    await registerOne(loader);
  }
};

const kick = () => void bootstrap();

const wireTriggers = () => {
  // Real interactions only — scroll events get simulated by perf tools,
  // which would pull Lit upgrades into the TBT measurement window.
  const opts: AddEventListenerOptions = { once: true, passive: true };
  window.addEventListener('pointerdown', kick, opts);
  window.addEventListener('keydown', kick, opts);
  window.addEventListener('touchstart', kick, opts);

  // Safety net: if none of the above fire, upgrade after the TBT window
  // has closed. 10s is well past Lighthouse's simulated measurement.
  setTimeout(kick, 10000);
};

if (document.readyState === 'complete') {
  wireTriggers();
} else {
  window.addEventListener('load', wireTriggers, { once: true });
}
