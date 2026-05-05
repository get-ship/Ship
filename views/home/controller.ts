// Home view — proof of the new directory convention. Loading the
// controller side-effect-imports _components/home-hero/, which runs
// its `customElements.define()` registration. The visit counter
// signal lives in the controller scope (the legacy in-template
// `<state>` form was removed when authoring discipline moved every
// reactive declaration into the controller).

import { controller, signal } from '@ship/ui-runtime'
import type { Scope } from './page.sui'
import './_components/home-hero/home-hero.controller'

export default controller<Scope>({
  scope: () => ({ seen: signal(0) }),
})
