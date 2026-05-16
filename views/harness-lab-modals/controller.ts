import { controller, signal, chrome } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => {
    const d1 = signal(false)
    const d2 = signal(false)
    const d3 = signal(false)
    const alert = signal(false)
    const drawer = signal(false)

    return {
      d1, d2, d3, alert, drawer,

      goNav: (e: Event, path: string): void => { e.preventDefault(); chrome.navigate(path) },

      open: (n: 1 | 2 | 3): void => {
        if (n === 1) d1.set(true)
        else if (n === 2) d2.set(true)
        else d3.set(true)
      },
      close: (n: 1 | 2 | 3): void => {
        if (n === 1) d1.set(false)
        else if (n === 2) d2.set(false)
        else d3.set(false)
      },
      openAlert: (): void => alert.set(true),

      openChromeModal: async (): Promise<void> => {
        const choice = await chrome.modal.request({
          title: 'Confirm via chrome',
          description: 'This modal is requested through the chrome IPC API.',
          actions: [
            { label: 'cancel', cancel: true },
            { label: 'proceed', tone: 'primary' },
          ],
          dismissible: true,
        })
        chrome.notify({
          tone: choice === 'proceed' ? 'success' : 'muted',
          title: 'Modal closed',
          description: `Picked: ${choice}`,
          duration: 2400,
        })
      },
    }
  },

  derived: {
    depth: ({ d1, d2, d3 }) => (d1() ? 1 : 0) + (d2() ? 1 : 0) + (d3() ? 1 : 0),
  },
})
