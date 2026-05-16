import { controller, signal, derived, chrome } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type Priority = 'low' | 'normal' | 'high'
type Tone = 'primary' | 'success' | 'warning' | 'info' | 'accent'

interface Card {
  id: string
  title: string
  summary?: string
  owner: string
  priority: Priority
  tone: Tone
  tags: string[]
}
interface Column {
  id: string
  title: string
  tone: Tone
  cards: Card[]
}

const OWNERS = ['Aria', 'Beck', 'Cora', 'Dax', 'Ezra', 'Fin']
const TAGS = ['frontend', 'backend', 'design', 'docs', 'a11y', 'perf', 'bug', 'spike']
const PRIORITIES: Priority[] = ['low', 'normal', 'normal', 'normal', 'high']
const TONES: Tone[] = ['primary', 'success', 'warning', 'info', 'accent']

let nextId = 0
const mkId = (): string => `c${++nextId}`

const pick = <T>(arr: readonly T[]): T => arr[Math.floor(Math.random() * arr.length)]

const TITLES = [
  'Wire up SSE reconnect', 'Audit focus ring on dark', 'Migrate config to TOML',
  'Drop legacy patch path', 'Tokenize router segments', 'Profile bindFor at 10k',
  'Add gauge component', 'Sweep aria-labels', 'Resolve race in dispatch',
  'Re-baseline visual diffs', 'Centralize date format', 'Split kanban controller',
  'Replace findByTag', 'Document live-source pattern',
]

const SUMMARIES = [
  'reconnect on socket loss + replay last seq',
  'tighten contrast in --muted-foreground',
  'jsonc tolerates trailing commas',
  '',
  'pathname → tuple of strings',
  'measure mount + diff cost',
  'arcs respect reduced-motion',
  '',
  'serialized writes per workspace',
  'one shared snapshot per branch',
]

const mkCard = (): Card => ({
  id: mkId(),
  title: pick(TITLES),
  summary: pick(SUMMARIES) || undefined,
  owner: pick(OWNERS),
  priority: pick(PRIORITIES),
  tone: pick(TONES),
  tags: Array.from({ length: 1 + Math.floor(Math.random() * 2) }, () => pick(TAGS)),
})

const seed = (n: number): Card[] => Array.from({ length: n }, mkCard)

const initial = (): Column[] => [
  { id: 'todo',  title: 'To do',       tone: 'info',    cards: seed(5) },
  { id: 'doing', title: 'In progress', tone: 'warning', cards: seed(3) },
  { id: 'done',  title: 'Done',        tone: 'success', cards: seed(4) },
]

const moveBetween = (
  state: Column[],
  cardId: string,
  targetCol: string,
  beforeId: string | null,
): Column[] => {
  const next = state.map((c) => ({ ...c, cards: [...c.cards] }))
  let moving: Card | null = null
  for (const c of next) {
    const idx = c.cards.findIndex((x) => x.id === cardId)
    if (idx >= 0) { moving = c.cards.splice(idx, 1)[0]; break }
  }
  if (!moving) return state
  const target = next.find((c) => c.id === targetCol)
  if (!target) return state
  if (beforeId == null) {
    target.cards.push(moving)
  } else {
    const idx = target.cards.findIndex((c) => c.id === beforeId)
    target.cards.splice(idx < 0 ? target.cards.length : idx, 0, moving)
  }
  return next
}

export default controller<Scope>({
  scope: () => {
    const cols = signal<Column[]>(initial())
    const draggingId = signal<string | null>(null)
    const selectedId = signal<string | null>(null)

    const selected = derived<Card | null>(() => {
      const id = selectedId()
      if (!id) return null
      for (const col of cols()) {
        const card = col.cards.find((c) => c.id === id)
        if (card) return card
      }
      return null
    })

    return {
      cols,
      draggingId,
      selectedId,
      selected,

      goNav: (e: Event, path: string): void => { e.preventDefault(); chrome.navigate(path) },

      onDragStart: (e: DragEvent, id: string): void => {
        draggingId.set(id)
        if (e.dataTransfer) {
          e.dataTransfer.setData('text/plain', id)
          e.dataTransfer.effectAllowed = 'move'
        }
      },
      onDragEnd: (): void => draggingId.set(null),
      onDragOver: (e: DragEvent): void => {
        if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
      },
      onDrop: (e: DragEvent, col: string, beforeId: string | null): void => {
        const id = e.dataTransfer?.getData('text/plain') || draggingId()
        if (!id || id === beforeId) { draggingId.set(null); return }
        cols.set(moveBetween(cols(), id, col, beforeId))
        draggingId.set(null)
      },

      addRandom: (): void => {
        const c = cols()
        const col = pick(c)
        cols.set(c.map((x) => x.id === col.id ? { ...x, cards: [...x.cards, mkCard()] } : x))
      },

      chaos: (): void => {
        let state = cols()
        for (let i = 0; i < 20; i++) {
          const op = Math.random()
          if (op < 0.4) {
            const filled = state.filter((x) => x.cards.length > 0)
            if (filled.length === 0) continue
            const from = pick(filled)
            const card = pick(from.cards)
            const to = pick(state)
            const beforeId = to.cards.length > 0 ? pick(to.cards).id : null
            state = moveBetween(state, card.id, to.id, beforeId)
          } else if (op < 0.75) {
            const col = pick(state)
            state = state.map((x) => x.id === col.id ? { ...x, cards: [...x.cards, mkCard()] } : x)
          } else {
            const all = state.flatMap((x) => x.cards)
            if (all.length === 0) continue
            const victim = pick(all)
            state = state.map((col) => ({ ...col, cards: col.cards.filter((c) => c.id !== victim.id) }))
          }
        }
        cols.set(state)
      },

      reset: (): void => { nextId = 0; cols.set(initial()); selectedId.set(null) },
      openCard: (id: string): void => selectedId.set(id),
      closeDrawer: (): void => selectedId.set(null),

      editField: (field: keyof Card, value: string): void => {
        const id = selectedId()
        if (!id) return
        cols.set(cols().map((col) => ({
          ...col,
          cards: col.cards.map((c) => c.id === id ? { ...c, [field]: value } as Card : c),
        })))
      },

      deleteSelected: (): void => {
        const id = selectedId()
        if (!id) return
        cols.set(cols().map((col) => ({ ...col, cards: col.cards.filter((c) => c.id !== id) })))
        selectedId.set(null)
      },
    }
  },
  derived: {
    columns:    ({ cols }) => cols(),
    totalCards: ({ cols }) => cols().reduce((n, c) => n + c.cards.length, 0),
    drawerOpen: ({ selectedId }) => selectedId() != null,
    selectedCard:     ({ selected }) => selected(),
    selectedTitle:    ({ selected }) => selected()?.title ?? '',
    selectedSummary:  ({ selected }) => selected()?.summary ?? '',
    selectedOwner:    ({ selected }) => selected()?.owner ?? '',
    selectedPriority: ({ selected }) => selected()?.priority ?? 'normal',
  },
})
