import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-app-icon.sui'

type AppName =
  | 'dashboard' | 'pipelines' | 'agents' | 'skills' | 'git'
  | 'collections' | 'canvas' | 'settings' | 'chat' | 'events'
  | 'specs' | 'theme' | 'setup' | 'overview' | 'mesh'

const APP_GLYPH: Record<AppName, string> = {
  dashboard: 'layout-dashboard',
  pipelines: 'workflow',
  agents: 'bot',
  skills: 'sparkles',
  git: 'git-branch',
  collections: 'database',
  canvas: 'frame',
  settings: 'settings',
  chat: 'messages-square',
  events: 'radio',
  specs: 'file-text',
  theme: 'palette',
  setup: 'rocket',
  overview: 'compass',
  mesh: 'network',
}

const APP_ALIASES: Record<string, AppName> = {
  'theme-studio': 'theme',
}

const APP_NAMES = new Set([...Object.keys(APP_GLYPH), ...Object.keys(APP_ALIASES)])

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    const stopEffect = effect(() => {
      const name = scope.name()
      const label = scope.label()
      const resolved = (APP_ALIASES[name] ?? name) as AppName
      const known = APP_NAMES.has(name)
      const glyph = known ? APP_GLYPH[resolved] : 'dot'

      const icon = scope.glyphEl() as HTMLElement | null
      if (icon) icon.setAttribute('name', glyph)

      if (label) {
        host.setAttribute('role', 'img')
        host.setAttribute('aria-label', label)
        host.removeAttribute('aria-hidden')
      } else {
        host.setAttribute('aria-hidden', 'true')
        host.removeAttribute('role')
        host.removeAttribute('aria-label')
      }
    })

    return () => stopEffect()
  },
})
