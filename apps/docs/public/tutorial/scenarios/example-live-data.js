// Demo scenario — proves the SW intercepts SSE + REST.
// Used by the harness smoke test; safe to consult as a template
// when authoring a real Views step scenario.

export default {
  '/users': {
    initial: { kind: 'loading' },
    frames: [
      {
        afterMs: 600,
        payload: {
          kind: 'ok',
          value: { items: [{ id: 1, name: 'alpha' }, { id: 2, name: 'beta' }] },
        },
      },
      {
        afterMs: 2400,
        payload: {
          kind: 'ok',
          value: {
            items: [
              { id: 1, name: 'alpha' },
              { id: 2, name: 'beta' },
              { id: 3, name: 'gamma' },
            ],
          },
        },
      },
      {
        afterMs: 4800,
        payload: { kind: 'error', error: { message: 'connection lost' } },
      },
    ],
  },
  'api:/whoami': { name: 'demo-user', id: 'u_42' },
}
