import { controller } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

interface ChromeBar {
  open(mode?: string): void
}

interface WindowWithChrome extends Window {
  chrome?: { bar?: ChromeBar }
  ship?: { command?: () => void }
}

export default controller<Scope>({
  scope: () => ({}),
  on: {
    openAgent: () => {
      const w = window as WindowWithChrome
      if (w.chrome?.bar?.open) {
        w.chrome.bar.open('ai')
        return
      }
      // Fallback: the SUI shell exposes `ship.command()` which dispatches
      // the same open event that the cmdk listens for; in shell2 the AI
      // prefix is typed by the user after open.
      w.ship?.command?.()
      document.dispatchEvent(
        new CustomEvent('ship:bar:open', { detail: { mode: 'ai' }, bubbles: true }),
      )
    },
  },
})
