import { signal } from '@ship/ui-runtime'

export default function setup() {
  return { count: signal(0) }
}
