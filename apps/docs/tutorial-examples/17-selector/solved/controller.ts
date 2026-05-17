import { controller, signal, selector } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

const ROWS = Array.from({ length: 200 }, (_, i) => ({ id: i, label: `r${i}` }))

export default controller<Scope>({
  scope: () => {
    const rows = signal(ROWS)
    const selectedId = signal(0)
    const isSelected = selector(selectedId)

    return {
      rows, selectedId, isSelected,
      move: (delta: number): void => {
        const next = (selectedId() + delta + ROWS.length) % ROWS.length
        selectedId.set(next)
      },
    }
  },
})
