import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-studio-components.sui'
import { currentComponentTag } from '../route-tag'

// Side-effect imports register the section's child elements so the
// router's `<sui-components-list>` / `<sui-component-detail>` tags
// resolve when this element bundles into sui-studio.js. Each child lives
// in its own tag-named directory so the compiler builds it in component
// mode (stem === parent dir → customElements.define).
import '../sui-components-list/sui-components-list.sui'
import '../sui-component-detail/sui-component-detail.sui'

export default controller<Scope>({
  derived: {
    // A non-empty `<tag>` segment means "show the detail for this tag".
    // Read reactively from the URL (see route-tag.ts) rather than the
    // shell-seeded route param, which doesn't survive the cross-bundle
    // boundary. The detail element validates the tag against the live
    // catalog and renders a not-found state for unknown ids.
    detailTag: () => currentComponentTag(),
  },
})
