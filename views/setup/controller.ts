import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => ({
    step: signal<1 | 2 | 3 | 4>(1),
    wsMode: signal<'new' | 'existing'>('new'),
    wsName: signal(''),
    wsPath: signal(''),
    wsError: signal(''),
    provider: signal(''),
    apiKey: signal(''),
    providerTestOk: signal(false),
  }),
  on: {
    onWsModeChange: (s, e: Event) => {
      const detail = (e as CustomEvent).detail
      s.wsMode = detail?.name ?? 'new'
      s.wsError = ''
    },
    onWsMutationError: (s, e: Event) => {
      const detail = (e as CustomEvent).detail
      s.wsError = detail?.message ?? 'Could not create workspace'
    },
  },
})
