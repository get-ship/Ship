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
})
