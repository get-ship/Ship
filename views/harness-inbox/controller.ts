import { controller, signal, derived, chrome } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type Kind = 'mention' | 'reply' | 'job' | 'alert' | 'system'
type Tone = 'primary' | 'success' | 'warning' | 'info' | 'accent' | 'danger'
type FilterId = 'all' | 'unread' | 'mention' | 'job' | 'alert'

interface Item {
  id: string
  from: string
  subject: string
  preview: string
  kind: Kind
  tone: Tone
  toneBadge: 'primary' | 'success' | 'warning' | 'info' | 'accent' | 'danger' | 'muted'
  read: boolean
  ts: number
}

const ROW_H = 88
const BUFFER = 6

const NAMES = ['ada', 'beck', 'cora', 'dax', 'ezra', 'fin', 'gale', 'hex', 'iri', 'juno', 'kit', 'lin']
const SUBJECTS = [
  'kanban DnD drops the wrong card on Safari',
  'visual baseline diff — sui-rail dark',
  'pipeline mesh-02 failed in CI',
  'agent.profile.compiled took 1.2s',
  'docs / charter is stale on L2',
  'switch token rebalance ready for review',
  'event stream paused after suspend',
  '@beck — can you look at the focus trap?',
  'collection schema rejected by validator',
  're: timeout in chat-reply pump',
]
const PREVIEWS = [
  'reproduces on touch devices when the source column has scroll …',
  'small contrast drift after the OKLCH migration — needs new png',
  'log shows the dispatcher gave up after 3 retries; trace attached',
  'spike from 0.4 baseline. likely the new derived in agents/',
  'L2 sandboxed-by-default but the charter still says privileged',
  'token values stable across light / dark / dim; awaiting one sign-off',
  'browser tab returned from suspend and the SSE channel was 401',
  'one last lookahead for the modal stack; merge after',
  'unknown discriminator at root: kind. validator/specta drift?',
  'cancel-on-disconnect is firing before the inner future yields',
]

const KIND_TONE: Record<Kind, Item['toneBadge']> = {
  mention: 'primary',
  reply:   'info',
  job:     'success',
  alert:   'danger',
  system:  'muted',
}

const KIND_DOT: Record<Kind, Tone> = {
  mention: 'primary',
  reply:   'info',
  job:     'success',
  alert:   'danger',
  system:  'accent',
}

let nextId = 0
const mkId = (): string => `i${++nextId}`
const pick = <T>(arr: readonly T[]): T => arr[Math.floor(Math.random() * arr.length)]

const KINDS: Kind[] = ['mention', 'reply', 'reply', 'job', 'job', 'alert', 'system']

const mkItem = (now = Date.now()): Item => {
  const kind = pick(KINDS)
  return {
    id: mkId(),
    from: pick(NAMES),
    subject: pick(SUBJECTS),
    preview: pick(PREVIEWS),
    kind,
    tone: KIND_DOT[kind],
    toneBadge: KIND_TONE[kind],
    read: Math.random() < 0.4,
    ts: now,
  }
}

const seed = (n: number): Item[] => {
  const now = Date.now()
  return Array.from({ length: n }, (_, i) => mkItem(now - i * 1000 * 60))
}

const fmtTime = (ts: number): string => {
  const d = Math.max(0, Date.now() - ts)
  if (d < 60_000) return `${Math.floor(d / 1000)}s`
  if (d < 3_600_000) return `${Math.floor(d / 60_000)}m`
  if (d < 86_400_000) return `${Math.floor(d / 3_600_000)}h`
  return `${Math.floor(d / 86_400_000)}d`
}

export default controller<Scope>({
  scope: () => {
    const items = signal<Item[]>(seed(2_500))
    const filterId = signal<FilterId>('all')
    const scrollTop = signal(0)
    const viewportH = signal(640)
    const streaming = signal(false)
    const intervalMs = signal(420)
    let timer: ReturnType<typeof setInterval> | null = null
    let scrollEl: HTMLElement | null = null

    const filtered = derived<Item[]>(() => {
      const all = items()
      const f = filterId()
      if (f === 'all') return all
      if (f === 'unread') return all.filter((i) => !i.read)
      return all.filter((i) => i.kind === f)
    })

    const startIdx = derived(() =>
      Math.max(0, Math.floor(scrollTop() / ROW_H) - BUFFER))
    const visibleCount = derived(() =>
      Math.ceil(viewportH() / ROW_H) + BUFFER * 2)
    const endIdx = derived(() =>
      Math.min(filtered().length, startIdx() + visibleCount()))

    const windowed = derived(() => {
      const list = filtered()
      const s = startIdx()
      const e = endIdx()
      return list.slice(s, e).map((it) => ({ ...it, timeLabel: fmtTime(it.ts) }))
    })

    return {
      items, filterId, scrollTop, viewportH, streaming, intervalMs,
      filtered,
      startIdx, endIdx, windowed, visibleCount,

      goNav: (e: Event, path: string): void => { e.preventDefault(); chrome.navigate(path) },

      setFilter: (f: FilterId): void => { filterId.set(f); if (scrollEl) scrollEl.scrollTop = 0 },

      toggleRead: (id: string): void => {
        items.set(items().map((i) => i.id === id ? { ...i, read: !i.read } : i))
      },

      markAllRead: (): void => {
        items.set(items().map((i) => i.read ? i : { ...i, read: true }))
      },

      burst: (): void => {
        const now = Date.now()
        const fresh = Array.from({ length: 50 }, (_, k) => mkItem(now - k * 200))
        items.set([...fresh, ...items()])
      },

      toggleStream: (): void => {
        if (streaming()) {
          streaming.set(false)
          if (timer) { clearInterval(timer); timer = null }
        } else {
          streaming.set(true)
          timer = setInterval(() => {
            items.set([mkItem(), ...items()])
          }, intervalMs())
        }
      },

      bindScrollEl: (el: HTMLElement): void => { scrollEl = el },
    }
  },

  derived: {
    totalCount:    ({ items }) => items().length,
    filteredCount: ({ filtered }) => filtered().length,
    windowedItems: ({ windowed }) => windowed(),
    currentFilter: ({ filterId }) => filterId(),
    avgInterval:   ({ intervalMs }) => intervalMs(),
    topPad:        ({ startIdx }) => startIdx() * ROW_H,
    bottomPad: ({ filtered, endIdx }) =>
      Math.max(0, (filtered().length - endIdx()) * ROW_H),
    filters: ({ items }) => {
      const all = items()
      const by = (k: Kind): number => all.filter((i) => i.kind === k).length
      return [
        { id: 'all'     as FilterId, label: 'All',      icon: 'inbox',       count: all.length },
        { id: 'unread'  as FilterId, label: 'Unread',   icon: 'circle-dot',  count: all.filter((i) => !i.read).length },
        { id: 'mention' as FilterId, label: 'Mentions', icon: 'at-sign',     count: by('mention') },
        { id: 'job'     as FilterId, label: 'Jobs',     icon: 'workflow',    count: by('job') },
        { id: 'alert'   as FilterId, label: 'Alerts',   icon: 'alert-triangle', count: by('alert') },
      ]
    },
  },

  mount: (state, el) => {
    const scrollEl = el.querySelector('.hi-scroll') as HTMLElement | null
    if (!scrollEl) return
    type ScopeState = {
      bindScrollEl: (el: HTMLElement) => void
      scrollTop: number
      viewportH: number
    }
    const s = state as unknown as ScopeState
    s.bindScrollEl(scrollEl)
    s.viewportH = scrollEl.clientHeight

    let raf = 0
    const onScroll = (): void => {
      if (raf) return
      raf = requestAnimationFrame(() => {
        raf = 0
        s.scrollTop = scrollEl.scrollTop
      })
    }
    scrollEl.addEventListener('scroll', onScroll, { passive: true })

    const ro = new ResizeObserver(() => { s.viewportH = scrollEl.clientHeight })
    ro.observe(scrollEl)

    return () => {
      scrollEl.removeEventListener('scroll', onScroll)
      if (raf) cancelAnimationFrame(raf)
      ro.disconnect()
    }
  },
})
