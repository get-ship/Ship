import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => ({
    step: signal<1 | 2 | 3 | 4>(1),
    wsMode: signal<'new' | 'existing'>('new'),
    wsName: signal(''),
    wsPath: signal(''),
    wsError: signal(''),
    wsCreating: signal(false),
    provider: signal(''),
    apiKey: signal(''),
    providerTestOk: signal(false),
    providerTesting: signal(false),
    providerTestError: signal(''),
    settingsSaving: signal(false),
  }),
  on: {
    onWsModeChange: (s, e: Event) => {
      const detail = (e as CustomEvent).detail
      s.wsMode = detail?.name ?? 'new'
      s.wsError = ''
    },
    onWsMutationError: (s, e: Event) => {
      const detail = (e as CustomEvent).detail
      s.wsError = detail?.message ?? detail?.error?.message ?? 'Could not create workspace'
      s.wsCreating = false
    },
    onWsSubmit: (s) => { s.wsCreating = true; s.wsError = '' },
    onWsSuccess: (s) => { s.wsCreating = false; s.step = 3; s.wsError = '' },
    onProviderTestSubmit: (s) => { s.providerTesting = true; s.providerTestError = '' },
    onProviderTestSuccess: (s) => {
      s.providerTesting = false
      s.providerTestOk = true
      s.providerTestError = ''
    },
    onProviderTestError: (s, e: Event) => {
      const detail = (e as CustomEvent).detail
      s.providerTesting = false
      s.providerTestOk = false
      s.providerTestError = detail?.error?.message ?? 'Connection failed'
    },
    onSettingsSubmit: (s) => { s.settingsSaving = true },
    onSettingsSuccess: (s) => { s.settingsSaving = false; s.step = 4 },
    onSettingsError: (s) => { s.settingsSaving = false },
  },
})
