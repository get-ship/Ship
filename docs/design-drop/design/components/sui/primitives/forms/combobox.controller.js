// combobox.controller.js — runtime twin of sui-combobox.controller.ts (types
// stripped; logic verbatim from madvib/Ship@job/sui-framework). No tsc needed.
// Pairs with combobox.sui via the workbench `setupUrl`.
import { controller, derived, effect, ref, signal } from '@ship/sui-runtime';

let listIdSeq = 0;

const parseJsonOptions = (raw) => {
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.map((entry) => {
      if (entry && typeof entry === 'object') {
        const value = entry.value != null ? String(entry.value) : '';
        const label = entry.label != null ? String(entry.label) : value;
        return value ? { value, label } : null;
      }
      if (typeof entry === 'string') return { value: entry, label: entry };
      return null;
    }).filter((o) => o !== null);
  } catch { return []; }
};

const readLightOptions = (host) =>
  Array.from(host.querySelectorAll(':scope > option')).map((el) => ({
    value: el.value || (el.textContent || '').trim() || '',
    label: (el.textContent || '').trim() || el.value || '',
  })).filter((o) => o.value !== '');

export default controller({
  scope: ({ options }) => {
    const open = signal(false);
    const filterText = signal('');
    const highlight = signal(0);
    const slotOptions = signal([]);
    const inputEl = ref(null);
    const listEl = ref(null);
    const optionsSlot = ref(null);
    const listId = signal(`sui-combobox-list-${++listIdSeq}`);

    const parsedOptions = derived(() => {
      const fromAttr = parseJsonOptions(options());
      return fromAttr.length > 0 ? fromAttr : slotOptions();
    });
    const visibleOptions = derived(() => {
      const q = filterText().trim().toLowerCase();
      const all = parsedOptions();
      const matched = q ? all.filter((o) => o.label.toLowerCase().includes(q)) : all;
      return matched.map((o, idx) => ({ ...o, idx, id: `sui-combobox-opt-${idx}` }));
    });
    const activeId = derived(() => {
      if (!open()) return '';
      const list = visibleOptions();
      return (list[highlight()] && list[highlight()].id) || '';
    });

    return { open, filterText, highlight, slotOptions, inputEl, listEl, optionsSlot, listId, parsedOptions, visibleOptions, activeId };
  },

  mount(state, host) {
    let lastEmitted = null;

    const refreshSlotOptions = () => { state.slotOptions = readLightOptions(host); };
    refreshSlotOptions();
    const slot = state.optionsSlot;
    slot && slot.addEventListener('slotchange', refreshSlotOptions);

    const labelFor = (value) => {
      const opts = state.parsedOptions;
      const hit = opts.find((o) => o.value === value);
      return hit ? hit.label : value;
    };

    const stopValueSync = effect(() => {
      const v = state.value;
      if (!state.open) state.filterText = labelFor(v);
    });
    state.filterText = labelFor(state.value);

    const stopOpenAttr = effect(() => {
      if (state.open) host.setAttribute('data-open', '');
      else host.removeAttribute('data-open');
    });

    const inputEl = state.inputEl;
    if (!inputEl) return;

    const openList = () => {
      if (state.disabled) return;
      if (!state.open) { state.open = true; state.highlight = 0; state.filterText = ''; }
    };
    const closeList = (commitLabelFromValue = true) => {
      if (!state.open) return;
      state.open = false;
      if (commitLabelFromValue) state.filterText = labelFor(state.value);
    };
    const commit = (rec) => {
      if (!rec) return;
      state.value = rec.value;
      state.open = false;
      state.filterText = rec.label;
      if (lastEmitted !== rec.value) {
        lastEmitted = rec.value;
        host.dispatchEvent(new CustomEvent('sui-change', {
          detail: { value: rec.value, label: rec.label }, bubbles: true, composed: true,
        }));
      }
    };

    const onFocus = () => openList();
    const onClick = () => openList();
    const onInput = () => { if (!state.open) state.open = true; state.highlight = 0; };
    const onKeydown = (e) => {
      const list = state.visibleOptions;
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        if (!state.open) { openList(); return; }
        if (list.length === 0) return;
        state.highlight = (state.highlight + 1) % list.length;
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        if (!state.open) { openList(); return; }
        if (list.length === 0) return;
        const h = state.highlight;
        state.highlight = h <= 0 ? list.length - 1 : h - 1;
      } else if (e.key === 'Enter') {
        if (!state.open) return;
        e.preventDefault();
        commit(list[state.highlight]);
      } else if (e.key === 'Escape') {
        if (!state.open) return;
        e.preventDefault();
        closeList(true);
      } else if (e.key === 'Tab') {
        closeList(true);
      }
    };
    const onBlur = (e) => {
      const next = e.relatedTarget;
      if (next && host.shadowRoot && host.shadowRoot.contains(next)) return;
      setTimeout(() => closeList(true), 0);
    };

    inputEl.addEventListener('focus', onFocus);
    inputEl.addEventListener('click', onClick);
    inputEl.addEventListener('input', onInput);
    inputEl.addEventListener('keydown', onKeydown);
    inputEl.addEventListener('blur', onBlur);

    const onShadowMousedown = (e) => {
      const li = e.target && e.target.closest && e.target.closest('li.opt[data-value]');
      if (!li) return;
      e.preventDefault();
      const value = li.dataset.value || '';
      commit(state.visibleOptions.find((o) => o.value === value));
    };
    host.shadowRoot && host.shadowRoot.addEventListener('mousedown', onShadowMousedown);

    if (host.expose) host.expose({ focus: () => inputEl.focus(), blur: () => inputEl.blur(), open: openList, close: () => closeList(true) });

    return () => {
      stopValueSync();
      stopOpenAttr();
      slot && slot.removeEventListener('slotchange', refreshSlotOptions);
      inputEl.removeEventListener('focus', onFocus);
      inputEl.removeEventListener('click', onClick);
      inputEl.removeEventListener('input', onInput);
      inputEl.removeEventListener('keydown', onKeydown);
      inputEl.removeEventListener('blur', onBlur);
      host.shadowRoot && host.shadowRoot.removeEventListener('mousedown', onShadowMousedown);
    };
  },
});
