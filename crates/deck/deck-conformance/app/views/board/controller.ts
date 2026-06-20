// Authored controller for the `board` control-flow proof — COMPOSED (imported
// + spread) by the synthetic controller, never rewritten.
//
// Phase-1 convention: the authored module default-exports a `() => scope`
// factory. The production `controller({ scope, derived, on })` DSL is richer;
// adapting that shape to the synthetic compose is a Phase-2 task (see the
// assessment). What the proof needs is that authored logic is composed in by
// import and type-checked alongside the template reads.
import { signal } from '@ship/sui-runtime'

export interface BoardItem {
  id: string
  label: string
  flagged: boolean
}

export default function boardScope() {
  const items = signal<BoardItem[]>([
    { id: 'a', label: 'Alpha', flagged: false },
    { id: 'b', label: 'Bravo', flagged: true },
  ])
  const add = (): void => {
    const n = items().length
    items.set([...items(), { id: `n${n}`, label: `Item ${n}`, flagged: false }])
  }
  const remove = (id: string): void => {
    items.set(items().filter((i) => i.id !== id))
  }
  return { items, add, remove }
}
