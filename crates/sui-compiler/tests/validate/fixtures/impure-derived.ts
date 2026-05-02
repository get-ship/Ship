import { controller } from '@ship/ui-runtime'

export default controller({
  derived: {
    x: () => Date.now(),
  },
})
