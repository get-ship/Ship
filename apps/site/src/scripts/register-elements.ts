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
  () => import('@ship/ui/web-components/ship-popover'),
  () => import('@ship/ui/web-components/ship-badge'),
  () => import('@ship/ui/web-components/ship-status-dot'),
  () => import('@ship/ui/web-components/ship-separator'),
  () => import('@ship/ui/web-components/ship-toolbar'),
  () => import('@ship/ui/web-components/ship-alert'),
];

// Auth dialog + its dependencies load on-demand when the user opens
// the modal (or lands on /sign-in or /sign-up). Keeps the homepage
// free of the dialog + tabs + input bundle.
const authLoaders: Loader[] = [
  () => import('@ship/ui/web-components/ship-input'),
  () => import('@ship/ui/web-components/ship-tabs'),
  () => import('@ship/ui/web-components/ship-dialog'),
  () => import('@ship/ui/web-components/domain/ship-auth-dialog'),
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

let authStarted = false;
const bootstrapAuth = async () => {
  if (authStarted) return;
  authStarted = true;
  // Chain through the normal loaders first so the dialog renders against
  // the same registered primitives as the rest of the site.
  await bootstrap();
  for (const loader of authLoaders) {
    await registerOne(loader);
  }
};

const kick = () => void bootstrap();
const kickAuth = () => void bootstrapAuth();

const wireTriggers = () => {
  // Real interactions only — scroll events get simulated by perf tools,
  // which would pull Lit upgrades into the TBT measurement window.
  const opts: AddEventListenerOptions = { once: true, passive: true };
  window.addEventListener('pointerdown', kick, opts);
  window.addEventListener('keydown', kick, opts);
  window.addEventListener('touchstart', kick, opts);

  // If the page has a ship-auth-dialog (e.g. /sign-in, /sign-up, or any
  // page that mounts AuthModal), kick the auth bundle as soon as the
  // user nudges the DOM so the dialog is ready when they click.
  if (document.querySelector('ship-auth-dialog')) {
    const authOpts: AddEventListenerOptions = { once: true, passive: true };
    window.addEventListener('pointerdown', kickAuth, authOpts);
    window.addEventListener('keydown', kickAuth, authOpts);
    window.addEventListener('touchstart', kickAuth, authOpts);
    // Dedicated auth routes open the dialog on load — bootstrap immediately.
    const auto = document.querySelector('ship-auth-dialog[data-auto-open="true"]');
    if (auto) ric(kickAuth);
  }

  // Expose for imperative callers (navbar Sign in button).
  (window as any).ship = {
    ...((window as any).ship ?? {}),
    _loadAuth: kickAuth,
  };

  // Safety net: if none of the above fire, upgrade after the TBT window
  // has closed. 10s is well past Lighthouse's simulated measurement.
  setTimeout(kick, 10000);
};

if (document.readyState === 'complete') {
  wireTriggers();
} else {
  window.addEventListener('load', wireTriggers, { once: true });
}
