// Hub home controller — the simplest possible controller. The
// `name` signal exists so the template has a signal to read,
// which triggers the compiler to emit a `hydrate()` function
// on the view's setup return. Without any signal read, the
// compiler doesn't emit hydrate and the runtime's mount() has
// nothing to call.
import { controller, signal, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => ({
    name: signal('Hub'),
  }),
})
