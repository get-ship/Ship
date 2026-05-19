import { controller, derived, effect, ref, signal } from '@ship/sui-runtime'
import type { Scope } from './sui-command-palette.sui'

/** One selectable row. `value` is opaque to the palette — the consumer
 *  decides what it means (a route, a `run:agent:<id>` token, etc.). */
export interface CommandItem {
  id: string
  label: string
  icon?: string
  hotkey?: string
  meta?: string
  value?: string
}

/** A labelled cluster of rows (Open · Run · Settings, …). */
export interface CommandGroup {
  label: string
  items: CommandItem[]
}

interface RenderItem extends CommandItem {
  rowId: string
  hint: string
  flatIdx: number
}

/** A single rendered line — either a group header or a selectable row.
 *  The list is flattened to one array so the template can drive it with a
 *  single keyed `{#for}`; nested reactive loops don't re-run their inner
 *  body when the outer key is unchanged, which broke live filtering. */
type RenderRow =
  | { kind: 'header'; key: string; label: string }
  | ({ kind: 'item'; key: string } & RenderItem)

let listIdSeq = 0

const matches = (it: CommandItem, q: string): boolean => {
  if (!q) return true
  const hay = `${it.label} ${it.meta ?? ''}`.toLowerCase()
  return hay.includes(q)
}

export default controller<Scope>({
  scope: () => {
    const query = signal('')
    const highlight = signal(0)
    const commands = signal<CommandGroup[]>([])
    const dialogEl = ref<HTMLDialogElement>(null)
    const inputEl = ref<HTMLInputElement>(null)
    const listEl = ref<HTMLDivElement>(null)
    const listId = signal(`sui-cmdp-list-${++listIdSeq}`)
    return { query, highlight, commands, dialogEl, inputEl, listEl, listId }
  },

  derived: {
    // Filter each group, drop empties, flatten to header+item rows, and
    // stamp a running flat index so arrow-key navigation can address every
    // visible item by one integer.
    rows: ({ commands, query }): RenderRow[] => {
      const q = query().trim().toLowerCase()
      const out: RenderRow[] = []
      let flat = 0
      for (const g of commands() as CommandGroup[]) {
        const matched = g.items.filter((it) => matches(it, q))
        if (!matched.length) continue
        out.push({ kind: 'header', key: `h:${g.label}`, label: g.label })
        for (const it of matched) {
          out.push({
            kind: 'item',
            key: `i:${it.id}`,
            ...it,
            rowId: `sui-cmdp-row-${flat}`,
            hint: it.hotkey ?? it.meta ?? '',
            flatIdx: flat,
          })
          flat += 1
        }
      }
      return out
    },
    flatCount: ({ rows }): number =>
      (rows() as RenderRow[]).filter((r) => r.kind === 'item').length,
    activeId: ({ rows, highlight }): string => {
      const h = highlight()
      const hit = (rows() as RenderRow[]).find((r) => r.kind === 'item' && r.flatIdx === h)
      return hit && hit.kind === 'item' ? hit.rowId : ''
    },
    resultLabel: ({ flatCount, query }): string => {
      const n = flatCount()
      const q = query().trim()
      if (!q) return ''
      return `${n} ${n === 1 ? 'result' : 'results'}`
    },
  },

  mount(state, host) {
    const dialogEl = state.dialogEl()
    if (!dialogEl) return

    // Flat list of every currently-visible item, in render order — the
    // single source the keyboard handlers index into.
    const flat = (): RenderItem[] =>
      (state.rows() as RenderRow[]).filter((r): r is RenderRow & { kind: 'item' } => r.kind === 'item')

    const close = (emit: boolean) => {
      if (emit) host.dispatchEvent(new CustomEvent('sui-command-close', { bubbles: true, composed: true }))
      if (state.open()) state.open.set(false)
    }

    const activate = (it: RenderItem | undefined) => {
      if (!it) return
      host.dispatchEvent(
        new CustomEvent('sui-command-select', {
          detail: { id: it.id, value: it.value ?? it.id },
          bubbles: true,
          composed: true,
        }),
      )
      state.open.set(false)
    }

    // Native showModal() drives the top layer + ::backdrop; the `open`
    // signal is the source of truth and this effect keeps the element
    // in sync in both directions' initiating edge.
    const stopOpenSync = effect(() => {
      const wantOpen = state.open()
      if (wantOpen && !dialogEl.open) {
        state.query.set('')
        state.highlight.set(0)
        try { dialogEl.showModal() } catch { /* jsdom lacks showModal */ }
        queueMicrotask(() => state.inputEl()?.focus())
      } else if (!wantOpen && dialogEl.open) {
        dialogEl.close()
      }
    })

    // Keep the highlight in range as the filter narrows the list.
    const stopClampHighlight = effect(() => {
      const n = state.flatCount()
      if (n === 0) { if (state.highlight() !== 0) state.highlight.set(0); return }
      if (state.highlight() > n - 1) state.highlight.set(n - 1)
    })

    // Typing resets the highlight to the first match.
    const stopQueryReset = effect(() => {
      state.query()
      state.highlight.set(0)
    })

    const onNativeClose = () => {
      if (state.open()) state.open.set(false)
    }

    const onKeydown = (e: KeyboardEvent) => {
      const list = flat()
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        if (list.length) state.highlight.set((state.highlight() + 1) % list.length)
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        if (list.length) {
          const h = state.highlight()
          state.highlight.set(h <= 0 ? list.length - 1 : h - 1)
        }
      } else if (e.key === 'Enter') {
        e.preventDefault()
        activate(list[state.highlight()])
      } else if (e.key === 'Escape') {
        // Let the native dialog close too, but emit our close intent.
        close(true)
      }
    }

    // Backdrop click — the click lands on the <dialog> itself when the
    // user hits the dimmed margin around the panel.
    const onDialogClick = (e: MouseEvent) => {
      if (e.target === dialogEl) close(true)
    }

    // A row was clicked. Rows are <button> with data-flat — resolve back
    // to the item via the flat list.
    const onRowClick = (e: MouseEvent) => {
      const row = (e.target as HTMLElement | null)?.closest?.('button.row[data-flat]') as HTMLElement | null
      if (!row) return
      const idx = Number(row.dataset.flat)
      activate(flat().find((it) => it.flatIdx === idx))
    }

    dialogEl.addEventListener('close', onNativeClose)
    dialogEl.addEventListener('keydown', onKeydown)
    dialogEl.addEventListener('click', onDialogClick)
    dialogEl.addEventListener('click', onRowClick)

    // `commands` JS property — structured data set from script (the shell
    // populates the dynamic Run group lazily on open). Backed by the
    // scope signal so writes re-render reactively.
    Object.defineProperty(host, 'commands', {
      configurable: true,
      get: () => state.commands(),
      set: (v: CommandGroup[]) => state.commands.set(Array.isArray(v) ? v : []),
    })

    // Exposed as `show`/`hide` (not `open`/`close`) so the method names
    // don't shadow the reflected `open` boolean property accessor.
    host.expose!({
      show: () => state.open.set(true),
      hide: () => close(false),
      focusInput: () => state.inputEl()?.focus(),
    })

    return () => {
      stopOpenSync()
      stopClampHighlight()
      stopQueryReset()
      dialogEl.removeEventListener('close', onNativeClose)
      dialogEl.removeEventListener('keydown', onKeydown)
      dialogEl.removeEventListener('click', onDialogClick)
      dialogEl.removeEventListener('click', onRowClick)
      try { delete (host as unknown as Record<string, unknown>).commands } catch { /* non-configurable */ }
    }
  },
})
