import { controller, signal } from '@ship/ui-runtime'

export default controller({
  scope: () => ({ count: signal(0) }),
})
