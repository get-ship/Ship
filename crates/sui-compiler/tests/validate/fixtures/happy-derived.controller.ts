import { controller } from '@ship/ui-runtime'

export default controller({
  derived: {
    doubled: ({ count }) => count() * 2,
    label: ({ count }) => `count is ${count()}`,
  },
  on: {
    inc: ({ count }) => count.set(count() + 1),
  },
})
