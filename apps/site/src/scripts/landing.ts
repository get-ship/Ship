/*
 * Landing-page behaviour.
 *
 * One script owns:
 *   - plate scene auto-morph (4.2s) + tab override + 1/2/3 keys
 *   - headline copy variants (4) with default "vibe"
 *   - vibe-chip swatches + `t` key: both go through @ship/ui/themes so
 *     localStorage stays canonical, then dispatch `ship-theme-change` so
 *     <ThemeSwitcher> updates its trigger label/active state.
 *   - tweaks panel (hidden; opened via Claude artifact postMessage)
 */

import {
  THEMES,
  applyTheme,
  persistTheme,
  readStoredTheme,
  isValidTheme,
  type ThemeId,
} from '@ship/ui/themes';

type SceneKey = 'session' | 'pipeline' | 'view';
type HeadlineKey = 'vibe' | 'build' | 'thirty' | 'reactive';
type CenterpieceKey = 'auto' | SceneKey;

const SCENES: SceneKey[] = ['session', 'pipeline', 'view'];
const AUTO_MS = 4200;

const HEADLINES: Record<HeadlineKey, { html: string; sub: string }> = {
  vibe: {
    html: `Vibe <span class="any">anything<span class="period">.</span></span>`,
    sub: `Ship lets you build <b>reactive AI applications</b> with advanced orchestration and custom data — <b>in about thirty seconds</b>. Local-first. Bring your own model.`,
  },
  build: {
    html: `Build AI apps<br/>that <span class="verb">feel<span class="period">.</span></span>`,
    sub: `Compose <b>agents, data collections, and themeable UI</b> the way you'd throw together a playlist. Ship runs it on your machine, on your model.`,
  },
  thirty: {
    html: `Thirty seconds<br/>to <span class="any">ship<span class="period">.</span></span>`,
    sub: `Define an <b>orchestration workflow</b>, pair it with your data, dress it in a theme. Ship turns that into a running, reactive app. Local-first.`,
  },
  reactive: {
    html: `The <span class="verb">reactive</span><br/><span class="any">AI runtime<span class="period">.</span></span>`,
    sub: `Ship is a <b>runtime and composer</b> for AI-native apps: orchestration, custom collections, themeable views. Local, fast, yours.`,
  },
};

interface State {
  headline: HeadlineKey;
  centerpiece: CenterpieceKey;
}
const state: State = { headline: 'vibe', centerpiece: 'auto' };

// ---- plate scenes --------------------------------------------------

const scenes = document.querySelectorAll<HTMLElement>('.scene');
const tabButtons = document.querySelectorAll<HTMLButtonElement>('.plate-tabs button[data-scene-btn]');
let autoTimer: number | null = null;

function showScene(name: SceneKey) {
  for (const s of scenes) s.classList.toggle('is-active', s.dataset.scene === name);
  for (const b of tabButtons) b.setAttribute('aria-selected', b.dataset.sceneBtn === name ? 'true' : 'false');
  const pill = document.querySelector<HTMLElement>('[data-theme-pill-label]');
  if (pill) pill.dataset.scene = name;
}

function stopAutoMorph() {
  if (autoTimer !== null) {
    window.clearInterval(autoTimer);
    autoTimer = null;
  }
}

function startAutoMorph() {
  stopAutoMorph();
  const activeScene = document.querySelector<HTMLElement>('.scene.is-active');
  let idx = Math.max(0, SCENES.indexOf((activeScene?.dataset.scene ?? 'session') as SceneKey));
  showScene(SCENES[idx]);
  autoTimer = window.setInterval(() => {
    idx = (idx + 1) % SCENES.length;
    showScene(SCENES[idx]);
  }, AUTO_MS);
}

function setCenterpiece(key: CenterpieceKey) {
  state.centerpiece = key;
  for (const b of document.querySelectorAll<HTMLButtonElement>('[data-tweak-group="centerpiece"] button')) {
    b.setAttribute('aria-pressed', b.dataset.centerpiece === key ? 'true' : 'false');
  }
  if (key === 'auto') startAutoMorph();
  else {
    stopAutoMorph();
    showScene(key);
  }
  persistEdit();
}

for (const b of tabButtons) {
  b.addEventListener('click', () => {
    const key = b.dataset.sceneBtn as SceneKey | undefined;
    if (key) setCenterpiece(key);
  });
}

// ---- headline ------------------------------------------------------

const headlineEl = document.getElementById('landing-headline');
const subEl = document.getElementById('landing-sub');

function setHeadline(key: HeadlineKey) {
  const h = HEADLINES[key] ?? HEADLINES.vibe;
  state.headline = key;
  if (headlineEl) headlineEl.innerHTML = h.html;
  if (subEl) subEl.innerHTML = h.sub;
  for (const b of document.querySelectorAll<HTMLButtonElement>('[data-tweak-group="headline"] button')) {
    b.setAttribute('aria-pressed', b.dataset.headline === key ? 'true' : 'false');
  }
  persistEdit();
}

for (const b of document.querySelectorAll<HTMLButtonElement>('[data-tweak-group="headline"] button')) {
  b.addEventListener('click', () => {
    const k = b.dataset.headline as HeadlineKey | undefined;
    if (k) setHeadline(k);
  });
}
for (const b of document.querySelectorAll<HTMLButtonElement>('[data-tweak-group="centerpiece"] button')) {
  b.addEventListener('click', () => {
    const k = b.dataset.centerpiece as CenterpieceKey | undefined;
    if (k) setCenterpiece(k);
  });
}

// ---- theme: vibe chip + t key -------------------------------------

const themeLabelEls = document.querySelectorAll<HTMLElement>('[data-theme-pill-label]');
const vibeSwatches = document.querySelectorAll<HTMLButtonElement>('[data-vibe-theme]');

function paintThemeLabel(id: ThemeId) {
  const theme = THEMES.find((t) => t.id === id);
  for (const el of themeLabelEls) el.textContent = theme?.id ?? id;
  for (const s of vibeSwatches) s.dataset.active = s.dataset.vibeTheme === id ? 'true' : 'false';
}

function setTheme(id: string) {
  if (!isValidTheme(id)) return;
  const next = applyTheme(id);
  persistTheme(next);
  paintThemeLabel(next);
  document.dispatchEvent(new CustomEvent('ship-theme-change', { detail: { id: next } }));
}

for (const s of vibeSwatches) {
  s.addEventListener('click', () => {
    const id = s.dataset.vibeTheme;
    if (id) setTheme(id);
  });
}

// Paint initial state from the stored value (the inline FOUC script has
// already applied the theme to <html>; we just reflect it in UI).
paintThemeLabel(readStoredTheme());

// Keep pill + swatches in sync if the ThemeSwitcher fires a change.
document.addEventListener('ship-theme-change', (e) => {
  const id = (e as CustomEvent<{ id: ThemeId }>).detail?.id;
  if (id) paintThemeLabel(id);
});

// ---- keyboard shortcuts -------------------------------------------

document.addEventListener('keydown', (e) => {
  const target = e.target as HTMLElement | null;
  if (target && target.matches('input, textarea, [contenteditable="true"]')) return;
  if (e.metaKey || e.ctrlKey || e.altKey) return;

  if (e.key === '1') setCenterpiece('session');
  else if (e.key === '2') setCenterpiece('pipeline');
  else if (e.key === '3') setCenterpiece('view');
  else if (e.key === 't' || e.key === 'T') {
    const ids = THEMES.map((t) => t.id);
    const current = readStoredTheme();
    const next = ids[(ids.indexOf(current) + 1) % ids.length];
    setTheme(next);
  }
});

// ---- tweaks panel + edit-mode postMessage handshake ---------------

const tweaksPanel = document.querySelector<HTMLElement>('[data-tweaks-panel]');
const tweaksClose = document.querySelector<HTMLButtonElement>('[data-tweaks-close]');

function setTweaksOpen(open: boolean) {
  if (!tweaksPanel) return;
  tweaksPanel.dataset.open = open ? 'true' : 'false';
  if (open) tweaksPanel.removeAttribute('hidden');
  else tweaksPanel.setAttribute('hidden', '');
}

tweaksClose?.addEventListener('click', () => setTweaksOpen(false));

window.addEventListener('message', (e) => {
  const d = e.data as { type?: string } | null;
  if (!d || typeof d !== 'object') return;
  if (d.type === '__activate_edit_mode') setTweaksOpen(true);
  else if (d.type === '__deactivate_edit_mode') setTweaksOpen(false);
});

function persistEdit() {
  try {
    window.parent.postMessage(
      {
        type: '__edit_mode_set_keys',
        edits: { headline: state.headline, centerpiece: state.centerpiece },
      },
      '*',
    );
  } catch {
    // No parent / cross-origin blocked — tweaks panel is cosmetic only.
  }
}

try {
  window.parent.postMessage({ type: '__edit_mode_available' }, '*');
} catch {
  // ignore
}

// ---- init ----------------------------------------------------------

setHeadline(state.headline);
setCenterpiece(state.centerpiece);
