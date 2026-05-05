import { controller, signal } from '@ship/ui-runtime'

export default controller({
  scope: () => ({ n: signal(0) }),
})
