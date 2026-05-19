import { controller, route } from '@ship/sui-runtime'
import type { Scope } from './sui-studio-themes.sui'
import { isThemePack } from '../theme-registry'

// Side-effect imports register the section's child elements so the
// router's `<sui-themes-list>` / `<sui-theme-detail>` tags resolve when
// this element bundles into sui-studio.js. Each child lives in its own
// tag-named directory so the compiler builds it in component mode
// (stem === parent dir → customElements.define).
import '../sui-themes-list/sui-themes-list.sui'
import '../sui-theme-detail/sui-theme-detail.sui'

// Lazy global route instance — deferred past module eval so SSR import of
// the controller doesn't touch `window`. The shell registers the
// `/studio/:section/:sub` shapes, so `params.sub` resolves the theme id.
let _route: ReturnType<typeof route> | null = null
const rt = (): ReturnType<typeof route> => (_route ??= route())

export default controller<Scope>({
  derived: {
    detailId: () => {
      const sub = rt().params.sub()
      return isThemePack(sub) ? sub : ''
    },
  },
})
