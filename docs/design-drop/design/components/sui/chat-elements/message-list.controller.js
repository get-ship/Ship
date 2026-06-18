// message-list.controller.js — design-pass controller for sui-message-list.
// `items` JS-property + sticky-bottom auto-scroll. Production swaps in the
// virtualized controller (createVirtualizer) behind the same `items` contract.
import { controller, effect, ref, signal } from '@ship/sui-runtime';

export default controller({
  scope: () => ({ items: signal([]), scrollEl: ref(null) }),
  mount(state, host) {
    const init = host.items;
    if (init != null) state.items = init;
    Object.defineProperty(host, 'items', {
      configurable: true,
      get: () => state.items,
      set: (v) => { state.items = v; },
    });

    let pinned = true;
    const atBottom = (el) => el.scrollHeight - el.scrollTop - el.clientHeight <= 48;
    const stopBind = effect(() => {
      const el = state.scrollEl;
      if (!el) return;
      const onScroll = () => { pinned = atBottom(el); };
      el.addEventListener('scroll', onScroll, { passive: true });
      return () => el.removeEventListener('scroll', onScroll);
    });

    const stopAuto = effect(() => {
      const n = state.items.length; // track
      const el = state.scrollEl;
      if (!el || !state.stickyBottom || !pinned || n === 0) return;
      queueMicrotask(() => { el.scrollTop = el.scrollHeight; });
    });

    return () => { stopBind(); stopAuto(); };
  },
});
