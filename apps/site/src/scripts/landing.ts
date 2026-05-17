/*
 * Landing-page behaviour.
 *
 * One script owns:
 *   - plate scene auto-morph (4.2s) + tab override + 1/2/3 keys
 *   - headline copy variants (4) with default "vibe"
 *   - vibe-chip swatches + `t` key: both go through ../lib/themes so
 *     localStorage stays canonical. applyTheme fires `ship:theme:change`
 *     which <sui-theme-picker> listens to for active-state sync.
 *   - tweaks panel (hidden; opened via Claude artifact postMessage)
 */

import {
  THEMES,
  applyTheme,
  persistTheme,
  readStoredTheme,
  isValidTheme,
  type ThemeId,
} from '../lib/themes';

type SceneKey = 'session' | 'view';
type HeadlineKey = 'vibe' | 'build' | 'thirty' | 'reactive';
type CenterpieceKey = 'auto' | SceneKey;

const SCENES: SceneKey[] = ['session', 'view'];
const AUTO_MS = 8000;

const HEADLINES: Record<HeadlineKey, { html: string; sub: string }> = {
  vibe: {
    html: `Vibe <span class="any">anything<span class="period">.</span></span>`,
    sub: `A local runtime for AI apps you compose from <b>agents, your data, and a real UI</b>. Bring your own model.`,
  },
  build: {
    html: `AI that <span class="verb">ships<span class="period">.</span></span>`,
    sub: `Drop in a skill, shape a collection, drop a view. Save the file. <b>No build step. No deploy. No SaaS.</b>`,
  },
  thirty: {
    html: `Personal AI,<br/>your <span class="any">way<span class="period">.</span></span>`,
    sub: `One config per agent. Your collections, your themes, your tools. Runs on your machine, with the model you choose.`,
  },
  reactive: {
    html: `The <span class="verb">reactive</span><br/><span class="any">AI runtime<span class="period">.</span></span>`,
    sub: `Agents, collections, views — every signal on one event bus. <b>Write a view; it lives.</b> Edit a skill; the agent picks it up.`,
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
const progressBar = document.querySelector<HTMLElement>('[data-plate-progress]');
const plateEl = document.querySelector<HTMLElement>('#landing-plate');
let autoTimer: number | null = null;
let progressStart = 0;
let progressRaf: number | null = null;
let isPaused = false;
let pausedAt = 0;

function showScene(name: SceneKey) {
  for (const s of scenes) s.classList.toggle('is-active', s.dataset.scene === name);
  for (const b of tabButtons) b.setAttribute('aria-selected', b.dataset.sceneBtn === name ? 'true' : 'false');
  const pill = document.querySelector<HTMLElement>('[data-theme-pill-label]');
  if (pill) pill.dataset.scene = name;
}

function tickProgress() {
  if (!progressBar) return;
  if (isPaused) { progressRaf = null; return; }
  const elapsed = Math.min(performance.now() - progressStart, AUTO_MS);
  progressBar.style.width = `${(elapsed / AUTO_MS) * 100}%`;
  progressRaf = requestAnimationFrame(tickProgress);
}

function resetProgress() {
  if (!progressBar) return;
  progressStart = performance.now();
  progressBar.style.width = '0%';
  if (progressRaf) cancelAnimationFrame(progressRaf);
  progressRaf = requestAnimationFrame(tickProgress);
}

function stopAutoMorph() {
  if (autoTimer !== null) {
    window.clearInterval(autoTimer);
    autoTimer = null;
  }
  if (progressRaf) {
    cancelAnimationFrame(progressRaf);
    progressRaf = null;
  }
  if (progressBar) progressBar.style.width = '0%';
}

function startAutoMorph() {
  stopAutoMorph();
  const activeScene = document.querySelector<HTMLElement>('.scene.is-active');
  let idx = Math.max(0, SCENES.indexOf((activeScene?.dataset.scene ?? 'session') as SceneKey));
  showScene(SCENES[idx]);
  resetProgress();
  autoTimer = window.setInterval(() => {
    if (isPaused) return;
    idx = (idx + 1) % SCENES.length;
    showScene(SCENES[idx]);
    resetProgress();
  }, AUTO_MS);
}

if (plateEl) {
  plateEl.addEventListener('mouseenter', () => {
    if (state.centerpiece !== 'auto') return;
    isPaused = true;
    pausedAt = performance.now();
    if (progressRaf) { cancelAnimationFrame(progressRaf); progressRaf = null; }
  });
  plateEl.addEventListener('mouseleave', () => {
    if (state.centerpiece !== 'auto') return;
    if (!isPaused) return;
    isPaused = false;
    // Resume the progress bar where it left off so the pause feels honest.
    progressStart += performance.now() - pausedAt;
    progressRaf = requestAnimationFrame(tickProgress);
  });
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

// ---- theme: plate pill label + t key ------------------------------

const themeLabelEls = document.querySelectorAll<HTMLElement>('[data-theme-pill-label]');

function paintThemeLabel(id: ThemeId) {
  for (const el of themeLabelEls) el.textContent = id;
}

function setTheme(id: string) {
  if (!isValidTheme(id)) return;
  const next = applyTheme(id);
  persistTheme(next);
  paintThemeLabel(next);
  // applyTheme already dispatches `ship:theme:change`, which the
  // <sui-theme-picker> listens to for active-state sync — no extra
  // event needed.
}

// Paint initial state from the stored value (the inline FOUC script has
// already applied the theme to <html>; we just reflect it in UI).
paintThemeLabel(readStoredTheme());

// Keep the plate pill in sync when any control (sui-theme-picker, this
// script's own `t` key) applies a theme.
document.addEventListener('ship:theme:change', (e) => {
  const id = (e as CustomEvent<{ id: ThemeId }>).detail?.id;
  if (id) paintThemeLabel(id);
});

// ---- keyboard shortcuts -------------------------------------------

document.addEventListener('keydown', (e) => {
  const target = e.target as HTMLElement | null;
  if (target && target.matches('input, textarea, [contenteditable="true"]')) return;
  if (e.metaKey || e.ctrlKey || e.altKey) return;

  if (e.key === '1') setCenterpiece('session');
  else if (e.key === '2') setCenterpiece('view');
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

// Skip initial setHeadline — the default copy is rendered server-side
// in Hero.astro to avoid an FOUC where the H1 is empty until JS runs.
// The headline tweak controls still call setHeadline on click.
setCenterpiece(state.centerpiece);
