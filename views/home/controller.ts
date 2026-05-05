// Home view — proof of the new directory convention. The visit counter
// initialised in page.sui's `<state>` is incremented by the inline handler;
// this controller exists to demonstrate the file shape (controller.ts, no
// stem prefix) and to register the view-private `<home-hero>` widget.
//
// Loading the controller side-effect-imports _components/home-hero/, which
// runs its `customElements.define()` registration. The router will move this
// to lazy-on-mount registration in a follow-on; for now an eager import is
// fine because the view itself is only loaded when the route resolves.

import './_components/home-hero/home-hero.controller'

export default null
