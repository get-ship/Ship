import { controller, route } from '@ship/sui-runtime'
import type { Scope } from './sui-studio-components.sui'

// Side-effect imports register the section's child elements so the
// router's `<sui-components-list>` / `<sui-component-detail>` tags
// resolve when this element bundles into sui-studio.js. Each child lives
// in its own tag-named directory so the compiler builds it in component
// mode (stem === parent dir → customElements.define).
import '../sui-components-list/sui-components-list.sui'
import '../sui-component-detail/sui-component-detail.sui'

// Lazy global route instance — deferred past module eval so SSR import
// of the controller doesn't touch `window`. The shell registers the
// `/studio/:section/:sub` shapes, so `params.sub` resolves the tag id of
// the component being inspected.
let _route: ReturnType<typeof route> | null = null
const rt = (): ReturnType<typeof route> => (_route ??= route())

export default controller<Scope>({
  derived: {
    // A non-empty `:sub` segment means "show the detail for this tag".
    // The detail element validates the tag against the live catalog and
    // renders a not-found state for unknown ids, so no validation here.
    detailTag: () => rt().params.sub() ?? '',
  },
})
