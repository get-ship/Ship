// Eager registration of the ship web components used on the marketing
// site. Earlier iterations gated this behind pointerdown + a 10s safety
// timeout to protect Lighthouse TBT, but that left fresh navigations
// stuck on the un-upgraded `:not(:defined)` fallback for up to ten
// seconds — spacing, paddings, and anything rendered from the shadow
// DOM snapped in late.
//
// We now upgrade as soon as the browser's `load` event fires (or
// immediately if the document is already complete, e.g. after a
// client-side navigation). All components ship in parallel so they
// register in one short burst rather than trickling through idle ticks.

type Loader = () => Promise<unknown>;

const coreLoaders: Loader[] = [
  () => import('@ship/ui/web-components/ship-button'),
  () => import('@ship/ui/web-components/ship-card'),
  () => import('@ship/ui/web-components/ship-popover'),
  () => import('@ship/ui/web-components/ship-badge'),
  () => import('@ship/ui/web-components/ship-status-dot'),
  () => import('@ship/ui/web-components/ship-separator'),
  () => import('@ship/ui/web-components/ship-toolbar'),
  () => import('@ship/ui/web-components/ship-alert'),
];

// Auth dialog + its deps load on first user interaction with an auth
// trigger (or immediately on dedicated /sign-in /sign-up routes).
// Not eager because they're below the fold / opt-in.
const authLoaders: Loader[] = [
  () => import('@ship/ui/web-components/ship-input'),
  () => import('@ship/ui/web-components/ship-tabs'),
  () => import('@ship/ui/web-components/ship-dialog'),
  () => import('@ship/ui/web-components/domain/ship-auth-dialog'),
];

const loadAll = (loaders: Loader[]) => Promise.all(loaders.map((l) => l()));

let corePromise: Promise<unknown> | null = null;
const bootstrap = () => (corePromise ??= loadAll(coreLoaders));

let authPromise: Promise<unknown> | null = null;
const bootstrapAuth = () =>
  (authPromise ??= bootstrap().then(() => loadAll(authLoaders)));

const wireAuth = () => {
  if (!document.querySelector('ship-auth-dialog')) return;
  const opts: AddEventListenerOptions = { once: true, passive: true };
  const kick = () => void bootstrapAuth();
  window.addEventListener('pointerdown', kick, opts);
  window.addEventListener('keydown', kick, opts);
  window.addEventListener('touchstart', kick, opts);
  if (document.querySelector('ship-auth-dialog[data-auto-open="true"]')) {
    void bootstrapAuth();
  }
};

const boot = () => {
  void bootstrap();
  wireAuth();
  (window as any).ship = {
    ...((window as any).ship ?? {}),
    _loadAuth: () => void bootstrapAuth(),
  };
};

if (document.readyState === 'complete') {
  boot();
} else {
  window.addEventListener('load', boot, { once: true });
}
