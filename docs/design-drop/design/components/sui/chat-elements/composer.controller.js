// composer.controller.js — runtime twin of sui-composer.controller.ts (types
// stripped; logic from madvib/Ship@job/sui-framework). Pairs with composer.sui.
import { controller, effect, ref, signal } from '@ship/sui-runtime';

export default controller({
  scope: () => {
    const text = signal('');
    const attachments = signal([]);
    const fieldEl = ref(null);
    const sendEl = ref(null);
    return { text, attachments, fieldEl, sendEl };
  },

  mount(state, host) {
    const init = host.attachments;
    if (init != null) state.attachments = init;
    Object.defineProperty(host, 'attachments', {
      configurable: true,
      get: () => state.attachments,
      set: (v) => { state.attachments = v; },
    });

    const submit = () => {
      if (state.disabled) return;
      const text = String(state.text || '').trim();
      if (text.length === 0) return;
      host.dispatchEvent(new CustomEvent('sui-composer-submit', {
        detail: { text, attachments: [...state.attachments] },
        bubbles: true, composed: true,
      }));
      state.text = '';
    };

    const onKeydown = (e) => {
      if (e.key !== 'Enter') return;
      if (state.multiline && e.shiftKey) return;
      e.preventDefault();
      submit();
    };

    const stopField = effect(() => {
      const field = state.fieldEl;
      if (!field) return;
      field.addEventListener('keydown', onKeydown);
      return () => field.removeEventListener('keydown', onKeydown);
    });
    const stopSend = effect(() => {
      const send = state.sendEl;
      if (!send) return;
      send.addEventListener('click', submit);
      return () => send.removeEventListener('click', submit);
    });

    return () => { stopField(); stopSend(); };
  },
});
