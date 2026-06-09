/*
 * desktop.ts — the landing-page desktop runtime.
 *
 * Owns window lifecycle (open / close / focus / drag / maximise), the
 * dock running-state dots, the "open them all" tour, the empty-desktop
 * hint, and the menubar clock. The site theme is handled by
 * <sui-theme-picker>; each window's own theme is fixed at author time
 * via data-theme-preview, so there's no theme wiring here.
 *
 * Disabled below 760px, where windows render as a static stacked feed.
 */

import { mountGrainient, type Grainient } from './grainient';
import { initSettings } from './settings';
import { hydrateWindowBody } from './window-lazy';

const MOBILE = '(max-width: 760px)';

interface DesktopState {
  topZ: number;
}

/* topZ starts BELOW the menubar (z-index 100) and stays there.
 * Previously it started AT 100 and incremented on every focus, so
 * windows climbed above the menubar and intercepted clicks meant for
 * the menubar items (SHIP brand, palette, Download). The menubar must
 * always win the click in the area it occupies. */
const state: DesktopState = { topZ: 50 };
const MENUBAR_Z = 100;

/* One nav: every window maps to a URL. Opening pushes history; back/
 * forward and deep links resolve to the right window. */
const PATHS: Record<string, string> = {
  welcome: '/', agents: '/agents', observability: '/observability',
  'custom-views': '/custom-views', security: '/security', pricing: '/pricing',
  data: '/data', orchestration: '/orchestration', spaces: '/spaces',
  docs: '/docs', arcade: '/arcade', download: '/download',
};
const APP_BY_PATH: Record<string, string> = Object.fromEntries(
  Object.entries(PATHS).map(([app, path]) => [path, app]),
);
let suppressUrl = false;

function pathFor(app: string): string { return PATHS[app] ?? '/'; }
function syncUrl(app: string): void {
  if (suppressUrl) return;
  const url = pathFor(app);
  if (location.pathname !== url) history.pushState({ app }, '', url);
}

const isMobile = () => window.matchMedia(MOBILE).matches;
const q = <T extends Element = HTMLElement>(sel: string, root: ParentNode = document) =>
  root.querySelector<T>(sel);
const qa = <T extends Element = HTMLElement>(sel: string, root: ParentNode = document) =>
  Array.from(root.querySelectorAll<T>(sel));

function win(app: string): HTMLElement | null {
  return q<HTMLElement>(`[data-app-window="${app}"]`);
}
function dockItem(app: string): HTMLElement | null {
  return q<HTMLElement>(`.dock-item[data-app-target="${app}"]`);
}

function focusWindow(w: HTMLElement): void {
  // Cap focused z-index BELOW the menubar so clicks in the menubar
  // strip always reach the menubar, never the window underneath.
  state.topZ = Math.min(state.topZ + 1, MENUBAR_Z - 1);
  w.style.zIndex = String(state.topZ);
  for (const other of qa<HTMLElement>('.os-window')) other.classList.remove('focused');
  w.classList.add('focused');
  syncFullscreenState();
}

function updateDock(): void {
  for (const item of qa<HTMLElement>('.dock-item[data-app-target]')) {
    const w = win(item.dataset.appTarget ?? '');
    item.classList.toggle('is-open', !!w?.classList.contains('open'));
  }
}

function updateHint(): void {
  const hint = q<HTMLElement>('.os-hint');
  hint?.classList.toggle('show', !q('.os-window.open'));
}

/* Per-window lazy init. Heavy work (IntersectionObservers, cycles,
 * wheel handlers, SVG SMIL pausing) only runs once the window first
 * opens — not at page load for every closed window. Tracked via a
 * WeakSet so we never bind twice. */
const inited = new WeakSet<HTMLElement>();

/* Closed windows ship their body inert in a <template data-window-body>.
 * On first open we clone it into place and run the window's deferred init
 * (whenWindowBody callbacks) before any wiring touches the content. */
function ensureWindowBody(w: HTMLElement): void {
  const tpl = w.querySelector<HTMLTemplateElement>('template[data-window-body]');
  if (!tpl) return;
  const content = w.querySelector('.content');
  if (content) content.insertBefore(tpl.content.cloneNode(true), tpl);
  tpl.remove();
  const app = w.dataset.appWindow;
  if (app) hydrateWindowBody(app);
}

function ensureWindowInit(w: HTMLElement): void {
  if (inited.has(w)) return;
  inited.add(w);
  wireRichWindow(w);
  // Only bind the popout wheel handler for windows that actually have one.
  if (w.querySelector('[data-popout]')) attachPopout(w);
}

function openWindow(app: string, isDefault = false): void {
  const w = win(app);
  if (!w) return;
  if (w.classList.contains('open')) {
    focusWindow(w);
    return;
  }
  // Single-window model. Opening any window closes every other one so
  // the desktop never stacks (Welcome floating over /observability was
  // the symptom of the old multi-window cascade — gone). The dock,
  // gallery, and URL are the navigation; the desktop shows one
  // surface at a time.
  for (const other of qa<HTMLElement>('.os-window.open')) {
    if (other === w) continue;
    other.classList.remove('open');
    pauseWindowMotion(other);
    stopRichWindow(other);
    inited.delete(other);
  }
  ensureWindowBody(w);
  ensureWindowInit(w);
  const dock = dockItem(app);
  if (dock && !isDefault && !isMobile()) {
    const ir = dock.getBoundingClientRect();
    w.style.transformOrigin =
      `${ir.left + ir.width / 2 - w.offsetLeft}px ${ir.top + ir.height / 2 - w.offsetTop}px`;
  } else {
    w.style.transformOrigin = '50% 110%';
  }
  requestAnimationFrame(() => {
    w.classList.add('open');
    focusWindow(w);
    resumeWindowMotion(w);
    updateDock();
    updateHint();
  });
  if (!isDefault) syncUrl(app);
}

/* CSS keyframes pause for free on `display: none` elements — that's
 * the only reason a closed window's nudge bob / hero bob isn't burning
 * CPU. SMIL `<animateMotion>` does NOT get this for free in Firefox;
 * a closed Orchestration window's 5 traveling dots keep timing.
 * `svg.pauseAnimations()` is the surgical fix — one call per SVG.
 * Important: we don't sprinkle CSS-side wildcard pause rules. A
 * `[data-paused] *` selector burns the style engine on every recalc. */
type SmilSvg = SVGSVGElement & {
  pauseAnimations?: () => void;
  unpauseAnimations?: () => void;
};
function pauseWindowMotion(w: HTMLElement): void {
  const svgs = w.querySelectorAll<SmilSvg>('svg');
  if (!svgs.length) return;
  for (const svg of Array.from(svgs)) svg.pauseAnimations?.();
}
function resumeWindowMotion(w: HTMLElement): void {
  const svgs = w.querySelectorAll<SmilSvg>('svg');
  if (!svgs.length) return;
  for (const svg of Array.from(svgs)) svg.unpauseAnimations?.();
}

const COMPACT_WINDOWS = new Set(['welcome', 'download']);

function closeWindow(w: HTMLElement): void {
  const dock = dockItem(w.dataset.appWindow ?? '');
  if (dock && !isMobile()) {
    const ir = dock.getBoundingClientRect();
    w.style.transformOrigin =
      `${ir.left + ir.width / 2 - w.offsetLeft}px ${ir.top + ir.height / 2 - w.offsetTop}px`;
  }
  w.classList.remove('open');
  pauseWindowMotion(w);
  // Tear down the window's IntersectionObserver + any cycling intervals.
  // Re-binding on the next open is cheap; carrying them while closed is
  // pure waste (and was the source of intervals running forever).
  stopRichWindow(w);
  inited.delete(w);
  updateDock();
  syncFullscreenState();
  const open = qa<HTMLElement>('.os-window.open')
    .sort((a, b) => (Number(b.style.zIndex) || 0) - (Number(a.style.zIndex) || 0));
  if (open[0]) focusWindow(open[0]);
  updateHint();
  syncUrl(open[0]?.dataset.appWindow ?? 'welcome');
}

function closeAll(): void {
  const open = qa<HTMLElement>('.os-window.open');
  for (const w of open) w.classList.remove('open');
  updateDock();
  updateHint();
  syncUrl('welcome');
}

/* Size toggle. Windows have two sizes: "large" (the default — generous,
 * uses available pixels, dock-safe) and "small" (compact stacking size).
 * Green traffic-light flips between them. Both modes use the cascade
 * anchors from Window.astro, just at different widths/heights. */
function toggleMax(w: HTMLElement): void {
  // Every window has both modes available. Green button flips.
  const current = w.dataset.size === 'mini' ? 'mini' : 'fullscreen';
  w.dataset.size = current === 'mini' ? 'fullscreen' : 'mini';
  syncFullscreenState();
}

function attachDrag(w: HTMLElement): void {
  const bar = w.querySelector<HTMLElement>('.titlebar');
  w.addEventListener('pointerdown', () => focusWindow(w), true);
  w.querySelector('.tl-close')?.addEventListener('click', (e) => { e.stopPropagation(); closeWindow(w); });
  w.querySelector('.tl-min')?.addEventListener('click', (e) => { e.stopPropagation(); closeWindow(w); });
  w.querySelector('.tl-max')?.addEventListener('click', (e) => { e.stopPropagation(); toggleMax(w); });
  if (!bar) return;

  /* Drag state machine:
   *
   *   IDLE        ──pointerdown──▶  ARMED
   *   ARMED       ──pointermove──▶  DRAGGING (commits inline styles)
   *   ARMED       ──pointerup────▶  IDLE   (was just a click, NO commit)
   *   DRAGGING    ──pointermove──▶  DRAGGING (updates left/top)
   *   DRAGGING    ──pointerup────▶  IDLE
   *
   * Why this exists: the previous implementation committed `left/top/
   * right:auto/bottom:auto` to inline styles on every pointerdown,
   * which broke `inset: 0` on fullscreen windows and collapsed them
   * to intrinsic content size — even on a plain click that never
   * moved the pointer. ARMED→DRAGGING gating prevents that.
   *
   * Also: fullscreen windows are never draggable. Dragging a
   * viewport-filling window has no meaning; let the green button
   * toggle to mini first if the user wants to move it.
   */
  let phase: 'idle' | 'armed' | 'dragging' = 'idle';
  let sx = 0, sy = 0, sl = 0, st = 0;
  bar.addEventListener('pointerdown', (e) => {
    if (isMobile() || (e.target as HTMLElement).closest('button, a, svg')) return;
    if (w.dataset.size === 'fullscreen') return;
    phase = 'armed';
    sx = e.clientX; sy = e.clientY;
    const r = w.getBoundingClientRect();
    sl = r.left; st = r.top;
    bar.setPointerCapture?.(e.pointerId);
  });
  bar.addEventListener('pointermove', (e) => {
    if (phase === 'idle') return;
    if (phase === 'armed') {
      // Need a few pixels of movement before we commit — that turns
      // arrest accidental drags from a click into a no-op.
      if (Math.abs(e.clientX - sx) < 3 && Math.abs(e.clientY - sy) < 3) return;
      // Now we're actually dragging. Commit the inline left/top once;
      // pointermove will update them from here.
      Object.assign(w.style, { left: `${sl}px`, top: `${st}px`, right: 'auto', bottom: 'auto' });
      phase = 'dragging';
    }
    // Top clamp = menubar + small gap. The window's titlebar must
    // remain reachable below the menubar — letting it tuck under makes
    // the traffic lights unclickable. Bottom clamp = measured dock
    // footprint so the window can't be dragged behind the dock either.
    const menu = document.querySelector<HTMLElement>('.os-menubar');
    const menuH = menu && getComputedStyle(menu).display !== 'none'
      ? Math.round(menu.getBoundingClientRect().height) : 0;
    const dock = document.querySelector<HTMLElement>('.os-dock');
    const dockH = dock && !dock.hidden ? Math.round(dock.getBoundingClientRect().height) : 0;
    const dockOffset = 14; /* matches Dock.astro `bottom:14px` */
    const bottomSafe = dockH > 0 ? dockH + dockOffset + 8 : 8;
    const minVisible = 40; /* keep at least one titlebar's worth of window visible */
    const topFloor = menuH + 4;
    const x = Math.max(2, Math.min(window.innerWidth - 80, sl + (e.clientX - sx)));
    const y = Math.max(topFloor, Math.min(window.innerHeight - bottomSafe - minVisible, st + (e.clientY - sy)));
    Object.assign(w.style, { left: `${x}px`, top: `${y}px` });
  });
  const stop = (e: PointerEvent) => {
    if (phase === 'idle') return;
    phase = 'idle';
    try { bar.releasePointerCapture?.(e.pointerId); } catch { /* released */ }
  };
  bar.addEventListener('pointerup', stop);
  bar.addEventListener('pointercancel', stop);
}

/* ── Rich-window behaviours: reveal, parallax, cycle, count-up. ──
 * Each window's scroll container is the IntersectionObserver root, so
 * sections reveal and counters fire as the *window* scrolls, not the
 * page. We stash the per-window cleanup hooks on the WeakMap so
 * close/reopen can pause/resume the intervals + observers instead of
 * letting them tick forever (they did before — the cycle interval
 * would run for the lifetime of the page once the window had been
 * opened once, even after close). */
interface RichHooks {
  io: IntersectionObserver;
  timers: number[];
}
const richHooks = new WeakMap<HTMLElement, RichHooks>();

function wireRichWindow(w: HTMLElement): void {
  const scroller = w.querySelector<HTMLElement>('.win-scroll');
  if (!scroller) return;

  const io = new IntersectionObserver((entries) => {
    for (const e of entries) {
      if (!e.isIntersecting) continue;
      e.target.classList.add('in');
      if (e.target instanceof HTMLElement && e.target.dataset.count) countUp(e.target);
      io.unobserve(e.target);
    }
  }, { root: scroller, threshold: 0.25 });
  for (const el of scroller.querySelectorAll('[data-reveal], [data-count]')) io.observe(el);

  const layers = scroller.querySelectorAll<HTMLElement>('[data-parallax]');
  if (layers.length) {
    // Write the parallax Y offset to a CSS custom property `--py` so
    // each component can compose it with its own transform (rotation,
    // position) instead of having `transform` clobbered. The CSS rule
    // `transform: translate3d(0, var(--py, 0), 0)` covers the simple
    // bg-only case (Pricing / Download); SecurityWindow combines --py
    // with --dx/--dy/--rot for its parallax threat field.
    scroller.addEventListener('scroll', () => {
      const t = scroller.scrollTop;
      for (const p of layers) {
        const f = Number(p.dataset.parallax) || 0.3;
        p.style.setProperty('--py', `${t * f}px`);
      }
    }, { passive: true });
  }

  const timers: number[] = [];
  for (const c of scroller.querySelectorAll<HTMLElement>('[data-cycle]')) {
    const id = startCycle(c);
    if (id) timers.push(id);
  }
  for (const g of scroller.querySelectorAll<HTMLElement>('[data-select]')) wireSelect(w, g);
  for (const b of scroller.querySelectorAll<HTMLElement>('[data-pause]')) wirePause(w, b);
  for (const r of scroller.querySelectorAll<HTMLElement>('[data-request]')) wireApproval(w, r);

  richHooks.set(w, { io, timers });
}

/* Called from closeWindow alongside pauseWindowMotion. Stops every
 * interval the window opened and disconnects its IntersectionObserver.
 * Doesn't tear down listeners or DOM — reopening the window just
 * re-binds via ensureWindowInit (now backed by a WeakSet, so it only
 * fires once; if you wanted to re-fire after close we'd need to clear
 * `inited` on close, but the intervals + IO are the costly part). */
function stopRichWindow(w: HTMLElement): void {
  const hooks = richHooks.get(w);
  if (!hooks) return;
  hooks.io.disconnect();
  for (const id of hooks.timers) clearInterval(id);
  hooks.timers.length = 0;
  richHooks.delete(w);
}

/* The Security approval card: Approve/Deny resolve the request in place
 * and prepend the decision to the audit table. */
function wireApproval(w: HTMLElement, req: HTMLElement): void {
  const acts = req.querySelector<HTMLElement>('[data-acts]');
  const resolved = req.querySelector<HTMLElement>('[data-resolved]');
  const audit = w.querySelector<HTMLElement>('[data-audit]');
  const who = req.dataset.who ?? 'Agent';
  const what = req.dataset.what ?? '';
  const decide = (ok: boolean) => {
    if (acts) acts.hidden = true;
    if (resolved) {
      resolved.hidden = false;
      resolved.className = `resolved ${ok ? 'ok' : 'no'}`;
      resolved.textContent = ok ? '✓ Approved · pushed' : '✕ Denied · nothing ran';
    }
    if (audit) {
      const tr = document.createElement('tr');
      tr.innerHTML =
        `<td class="c-dot"><sui-status-dot tone="${ok ? 'success' : 'destructive'}" size="sm"></sui-status-dot></td>` +
        `<td><b>${who}</b></td><td class="c-what">${what}${ok ? '' : ' — denied'}</td><td class="c-t">now</td>`;
      audit.prepend(tr);
    }
  };
  req.querySelector('[data-approve]')?.addEventListener('click', () => decide(true));
  req.querySelector('[data-deny]')?.addEventListener('click', () => decide(false));
}

/* A single-choice group of <sui-button data-select-item>. Selecting one
 * flips it to solid/primary (others outline) and writes its data-value
 * into the element named by data-select-output. Real toggle, no markup. */
function wireSelect(w: HTMLElement, group: HTMLElement): void {
  const items = Array.from(group.querySelectorAll<HTMLElement>('[data-select-item]'));
  const out = group.dataset.selectOutput ? w.querySelector<HTMLElement>(group.dataset.selectOutput) : null;
  const demo = group.dataset.demoTarget ? w.querySelector<HTMLElement>(group.dataset.demoTarget) : null;
  const apply = (chosen: HTMLElement) => {
    for (const it of items) {
      const on = it === chosen;
      it.setAttribute('variant', on ? 'solid' : 'outline');
      it.setAttribute('pressed', on ? 'true' : 'false');
      if (on) it.setAttribute('tone', 'primary'); else it.removeAttribute('tone');
    }
    const value = chosen.dataset.value;
    if (out && value) out.textContent = value;
    // Repaint a demo: retone its tone-driven primitives live.
    if (demo && value) {
      for (const el of demo.querySelectorAll('sui-bar-chart, sui-progress, sui-status-dot')) {
        el.setAttribute('tone', value);
      }
    }
  };
  for (const it of items) it.addEventListener('click', () => apply(it));
}

/* A <sui-button data-pause="<cycle selector>"> that freezes/resumes the
 * named [data-cycle] animation and flips its own label. */
function wirePause(w: HTMLElement, btn: HTMLElement): void {
  const target = btn.dataset.pause ? w.querySelector<HTMLElement>(btn.dataset.pause) : null;
  if (!target) return;
  btn.addEventListener('click', () => {
    const paused = target.classList.toggle('paused');
    btn.setAttribute('pressed', paused ? 'true' : 'false');
    btn.textContent = paused ? 'Paused' : 'Live';
  });
}

function countUp(el: HTMLElement): void {
  const target = Number(el.dataset.count) || 0;
  const dur = 900;
  const start = performance.now();
  const step = (now: number) => {
    const p = Math.min(1, (now - start) / dur);
    const eased = 1 - (1 - p) ** 3;
    el.textContent = Math.round(target * eased).toLocaleString();
    if (p < 1) requestAnimationFrame(step);
  };
  requestAnimationFrame(step);
}

/* Returns the interval handle so the caller can clear it on window
 * close. Was previously orphaning the timer — it ran forever, even
 * after the window closed, capturing items[] in a closure that the
 * GC couldn't collect. */
function startCycle(container: HTMLElement): number {
  const items = Array.from(container.querySelectorAll<HTMLElement>('[data-cycle-item]'));
  if (items.length < 2) return 0;
  const period = Number(container.dataset.cycle) || 1500;
  let i = items.findIndex((x) => x.classList.contains('active'));
  if (i < 0) i = 0;
  items[i].classList.add('active');
  return window.setInterval(() => {
    if (container.classList.contains('paused')) return;
    items[i].classList.remove('active');
    i = (i + 1) % items.length;
    items[i].classList.add('active');
  }, period);
}

/* Dock "Features" folder: a popover of labeled launchers. */
function closeFolder(): void {
  const f = q<HTMLElement>('[data-folder]');
  const t = q<HTMLElement>('[data-folder-toggle]');
  if (f) { f.classList.remove('open'); f.hidden = true; }
  t?.setAttribute('aria-expanded', 'false');
}
function wireFolder(): void {
  const toggle = q<HTMLElement>('[data-folder-toggle]');
  const folder = q<HTMLElement>('[data-folder]');
  if (!toggle || !folder) return;
  toggle.addEventListener('click', (e) => {
    e.stopPropagation();
    const willOpen = folder.hidden;
    folder.hidden = false;
    requestAnimationFrame(() => folder.classList.toggle('open', willOpen));
    toggle.setAttribute('aria-expanded', String(willOpen));
    if (!willOpen) closeFolder();
  });
  document.addEventListener('click', (e) => {
    if (!folder.classList.contains('open')) return;
    if (!(e.target as HTMLElement).closest('.folder-wrap')) closeFolder();
  });
  document.addEventListener('keydown', (e) => { if (e.key === 'Escape') closeFolder(); });
}

/* Rotate any [data-rotate] element (wallpaper tagline, hero word)
 * through its pipe-separated data-lines, fading between them.
 *
 * Bail when the element's host window is closed or the tab is hidden.
 * Was previously firing forever — including a nested 280ms setTimeout
 * inside the 2800ms interval — even when the Welcome window had been
 * dismissed and the rotating text was display:none. */
function rotateTagline(): void {
  for (const el of qa<HTMLElement>('[data-rotate]')) {
    const lines = (el.dataset.lines ?? '').split('|').filter(Boolean);
    if (lines.length < 2) continue;
    const host = el.closest<HTMLElement>('.os-window');
    let i = 0;
    setInterval(() => {
      if (document.hidden) return;
      if (host && !host.classList.contains('open')) return;
      el.style.opacity = '0';
      setTimeout(() => {
        i = (i + 1) % lines.length;
        el.textContent = lines[i];
        el.style.opacity = '';
      }, 280);
    }, 2800);
  }
}

/* The grainient WebGL wallpaper — mounted once, recoloured on theme change. */
let grain: Grainient | null = null;
function startGrainient(): void {
  const canvas = q<HTMLCanvasElement>('[data-grainient]');
  if (canvas && !grain) grain = mountGrainient(canvas);
  grain?.start();
  document.addEventListener('ship:theme:change', () => grain?.recolor());
}

function startClock(): void {
  const clock = q<HTMLElement>('[data-os-clock]');
  if (!clock) return;
  const tick = () => {
    const d = new Date();
    clock.textContent =
      `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
  };
  tick();
  setInterval(tick, 30_000);
}

/* Popout drawer: each feature window has a CTA drawer hidden at the
 * bottom of its content area. A scroll/wheel gesture (or the visible
 * nudge button) slides it up. The × closes it. Demos are designed to fit
 * the window — the popout is the *only* thing scroll reveals. */
function attachPopout(w: HTMLElement): void {
  const content = w.querySelector<HTMLElement>('.content');
  const popout = w.querySelector<HTMLElement>('[data-popout]');
  const openBtn = w.querySelector<HTMLElement>('[data-popout-open]');
  const closeBtn = w.querySelector<HTMLElement>('[data-popout-close]');
  if (!content || !popout) return;
  const open = (): void => {
    if (popout.hasAttribute('data-open')) return;
    popout.hidden = false;
    requestAnimationFrame(() => popout.setAttribute('data-open', ''));
    content.setAttribute('data-popout-shown', '');
  };
  const close = (): void => {
    popout.removeAttribute('data-open');
    content.removeAttribute('data-popout-shown');
    setTimeout(() => { popout.hidden = true; }, 360);
  };
  content.addEventListener('wheel', (e: WheelEvent) => {
    // The popout drawer is only the trigger in DRAWER MODE — large
    // desktop window, popout-enabled, size not 'small'. In SCROLL MODE
    // (mobile or small-window or no-popout) the page just scrolls;
    // we don't capture wheel and the popout is inline instead.
    if (isMobile() || w.dataset.size === 'small') return;
    // Desktop drawer: scroll DOWN opens, scroll UP closes. Threshold
    // both sides so a tiny touchpad twitch is ignored.
    const isOpen = popout.hasAttribute('data-open');
    if (e.deltaY > 12 && !isOpen) {
      e.preventDefault();
      open();
    } else if (e.deltaY < -12 && isOpen) {
      e.preventDefault();
      close();
    }
  }, { passive: false });
  openBtn?.addEventListener('click', open);
  closeBtn?.addEventListener('click', close);
}

/* Mobile app-switcher gallery — opens a full-screen overlay listing every
 * window as a tile. Tapping a tile opens that window and closes the
 * gallery. Triggered by the grid-icon button in the menubar (mobile-only
 * by default; desktop hides the button but the gallery still works). */
function attachSwitcher(): void {
  const switcher = q<HTMLElement>('[data-switcher]');
  if (!switcher) return;
  const open = (): void => { switcher.hidden = false; };
  const close = (): void => { switcher.hidden = true; };
  // There are now MANY [data-switcher-toggle] elements: one in the
  // menubar (visible in mini mode) and one in every fullscreen
  // window's .fs-tail. Wire all of them — q() found only the first
  // (menubar's), which was hidden in fullscreen, leaving the user
  // with no working gallery from inside a fullscreen view.
  for (const t of qa<HTMLElement>('[data-switcher-toggle]')) t.addEventListener('click', open);
  q<HTMLElement>('[data-switcher-close]')?.addEventListener('click', close);
  // Tap a tile → open that window AND close the gallery.
  for (const tile of qa<HTMLElement>('.os-switcher-tile[data-app-target]')) {
    tile.addEventListener('click', () => { close(); });
  }
  // Escape dismisses.
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && !switcher.hidden) close();
  });
}

/* The dock-safe area is defined by --os-dock-h. The default value in
 * landing-os.css is a sensible guess; this measures the actual rendered
 * dock and writes the truth onto :root so windows can size to it
 * precisely. Anything that changes the dock geometry (theme, tile size,
 * shadow) is now self-correcting without touching the CSS variable. */
/* body[data-fullscreen] is set whenever the focused window is in
 * `data-size='fullscreen'` mode. The standalone menubar hides
 * (window's titlebar absorbs its right cluster) and the dock
 * auto-hides. In `data-size='mini'` mode both bars + dock show
 * normally. */
function syncFullscreenState(): void {
  const focused = q<HTMLElement>('.os-window.focused.open');
  const isFullscreen = focused?.dataset.size === 'fullscreen';
  document.body.toggleAttribute('data-fullscreen', isFullscreen);
}

function init(): void {
  initSettings();
  // Mobile-only: Arcade is desktop-only (the game has fixed pixel
  // geometry that doesn't translate to mobile yet). If a user deep-
  // links to /arcade on a phone, bounce them home so they don't land
  // on a broken surface.
  if (isMobile() && location.pathname === '/arcade') {
    history.replaceState(null, '', '/');
    const arcade = win('arcade');
    if (arcade) arcade.classList.remove('open');
    const welcome = win('welcome');
    if (welcome) welcome.classList.add('open');
  }
  for (const w of qa<HTMLElement>('.os-window')) {
    // Drag wiring is small + needed before any open (a deep-linked
    // window should already be draggable). The expensive bits —
    // IntersectionObservers, popout wheel handler, cycles — defer to
    // first open via ensureWindowInit().
    attachDrag(w);
    if (w.classList.contains('open')) {
      // Server-rendered default for this route. We never got an
      // openWindow() call for it, so init in place — otherwise the
      // popout wheel handler + reveal observers never bind.
      ensureWindowInit(w);
    } else {
      // SMIL motion + CSS keyframes inside closed windows otherwise run
      // forever — Orchestration's 5 animateMotions + welcome bob were
      // burning CPU while sitting on /agents.
      pauseWindowMotion(w);
    }
  }
  attachSwitcher();
  // Theme is owned by <sui-theme-picker>: it persists to localStorage
  // and dispatches ship:theme:change. We just listen (the grainient
  // recolour subscribes inside startGrainient).
  // Dock tiles + folder launchers both use data-app-target.
  for (const item of qa<HTMLElement>('[data-app-target]')) {
    item.addEventListener('click', () => { openWindow(item.dataset.appTarget ?? ''); closeFolder(); });
  }
  // Any control can open a window; falls back to its href when the target
  // window isn't on this page (e.g. the menubar pill on a deep route).
  for (const el of qa<HTMLElement>('[data-open-window]')) {
    el.addEventListener('click', (e) => {
      const app = el.dataset.openWindow ?? '';
      if (win(app)) { e.preventDefault(); openWindow(app); }
    });
  }
  wireFolder();
  startGrainient();
  for (const el of qa<HTMLElement>('[data-close-all]')) el.addEventListener('click', closeAll);
  document.addEventListener('keydown', (e) => {
    if (e.key !== 'Escape' || q('[data-folder]:not([hidden])')) return;
    const focused = q<HTMLElement>('.os-window.focused.open');
    if (focused) closeWindow(focused);
  });
  window.addEventListener('popstate', () => {
    const app = APP_BY_PATH[location.pathname] ?? 'welcome';
    suppressUrl = true;
    openWindow(app, app === 'welcome');
    suppressUrl = false;
  });
  if (isMobile()) {
    updateDock();
    startClock();
    rotateTagline();
    return;
  }

  const defaults = qa<HTMLElement>('[data-default="true"]');
  defaults.forEach((w, i) =>
    setTimeout(() => openWindow(w.dataset.appWindow ?? '', true), 90 + i * 130));
  startClock();
  rotateTagline();
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
