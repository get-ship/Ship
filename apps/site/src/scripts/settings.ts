/*
 * settings.ts — site preferences. Persists to localStorage under
 * `ship-site-settings` (one JSON blob) and applies them as data-*
 * attributes on `<html>`. The settings popover (SettingsPopover.astro,
 * one instance in the menubar and one in every window's .fs-tail)
 * reads + writes via the same handlers — any instance can change the
 * setting and every other instance reflects it immediately via the
 * `ship:settings:change` event.
 *
 * Add a setting in three steps:
 *   1) Add a row to SettingsPopover.astro with `data-<setting>` buttons.
 *   2) Add the key to `DEFAULTS` here.
 *   3) Add a CSS rule on `:root[data-<setting>='value']` in landing-os.css.
 * No central enum; no JS style mutation.
 */

const STORAGE_KEY = 'ship-site-settings';

type DockPos = 'bottom' | 'left' | 'right';
interface Settings { dockPos: DockPos }
const DEFAULTS: Settings = { dockPos: 'bottom' };

function read(): Settings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as Partial<Settings>;
    return {
      dockPos: parsed.dockPos === 'left' || parsed.dockPos === 'right' ? parsed.dockPos : 'bottom',
    };
  } catch {
    return { ...DEFAULTS };
  }
}

function write(s: Settings): void {
  try { localStorage.setItem(STORAGE_KEY, JSON.stringify(s)); }
  catch { /* private mode / quota */ }
}

function apply(s: Settings): void {
  document.documentElement.setAttribute('data-dock-pos', s.dockPos);
}

function emit(s: Settings): void {
  document.dispatchEvent(new CustomEvent('ship:settings:change', { detail: s }));
}

/* Wire one settings popover instance to the live settings model. Each
 * instance has its own .seg-opt buttons and a toggle button. */
function wireInstance(wrap: HTMLElement, getState: () => Settings, setState: (next: Settings) => void): () => void {
  const toggle = wrap.querySelector<HTMLButtonElement>('[data-settings-toggle]');
  const panel  = wrap.querySelector<HTMLElement>('[data-settings-panel]');
  if (!toggle || !panel) return () => {};

  const opts = Array.from(wrap.querySelectorAll<HTMLButtonElement>('.seg-opt[data-dock-pos]'));

  function reflect(state: Settings): void {
    for (const o of opts) {
      const active = o.dataset.dockPos === state.dockPos;
      o.setAttribute('aria-checked', active ? 'true' : 'false');
    }
  }

  function openPanel(): void {
    panel!.hidden = false;
    toggle!.setAttribute('aria-expanded', 'true');
  }
  function closePanel(): void {
    panel!.hidden = true;
    toggle!.setAttribute('aria-expanded', 'false');
  }

  function onToggle(e: Event): void {
    e.stopPropagation();
    if (panel!.hidden) openPanel(); else closePanel();
  }
  toggle.addEventListener('click', onToggle);

  for (const o of opts) {
    o.addEventListener('click', () => {
      const pos = o.dataset.dockPos as DockPos;
      setState({ ...getState(), dockPos: pos });
    });
  }

  function onDoc(e: MouseEvent): void {
    if (panel!.hidden) return;
    if (!wrap.contains(e.target as Node)) closePanel();
  }
  document.addEventListener('click', onDoc);

  function onSettings(e: Event): void {
    reflect((e as CustomEvent<Settings>).detail);
  }
  document.addEventListener('ship:settings:change', onSettings);

  reflect(getState());

  return () => {
    toggle.removeEventListener('click', onToggle);
    document.removeEventListener('click', onDoc);
    document.removeEventListener('ship:settings:change', onSettings);
  };
}

/* Boot. Read from storage, apply, wire every instance. Returns the
 * apply function so callers can re-run it after late mounts. */
export function initSettings(): void {
  let state = read();
  apply(state);

  const setState = (next: Settings): void => {
    state = next;
    write(state);
    apply(state);
    emit(state);
  };

  for (const wrap of Array.from(document.querySelectorAll<HTMLElement>('.settings-wrap'))) {
    wireInstance(wrap, () => state, setState);
  }
}
