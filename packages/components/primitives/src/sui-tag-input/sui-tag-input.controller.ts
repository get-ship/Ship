import { controller, derived, effect, ref, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-tag-input.sui'

interface TagEntry { key: string; idx: number; label: string }

const parseCsv = (raw: string): string[] =>
  raw
    .split(',')
    .map((s) => s.trim())
    .filter((s) => s.length > 0)

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const tagList = signal<string[]>([])
    const entryValue = signal('')
    const inputEl = ref<HTMLInputElement>(null)
    const tags = derived<TagEntry[]>(() =>
      tagList().map((label, idx) => ({ key: `${idx}:${label}`, idx, label })),
    )
    return { tagList, entryValue, inputEl, tags }
  },

  mount(state, host) {
    // Without these, a screen reader announces the host as an
    // anonymous group of nested chips + an input — the user can't
    // tell it's one tag-input control with a list of accepted values.
    // role=group + a live aria-label summarising current state lets
    // SR users hear "Tags, 3 selected: docs, ship, view" instead of
    // a chip-by-chip walk.
    host.setAttribute('role', 'group')

    let suppressValueSync = false

    const emit = (next: string[]) => {
      host.dispatchEvent(
        new CustomEvent('sui-change', {
          detail: { tags: [...next] },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const writeBack = (next: string[]) => {
      suppressValueSync = true
      state.tagList = next
      state.value = next.join(',')
      suppressValueSync = false
      emit(next)
    }

    // value attr (CSV) → internal tag array. Skips the round-trip when
    // we just wrote it ourselves via writeBack().
    const stopValueSync = effect(() => {
      const raw = state.value
      if (suppressValueSync) return
      const parsed = parseCsv(raw)
      const cur = state.tagList
      if (parsed.length === cur.length && parsed.every((v, i) => v === cur[i])) return
      state.tagList = parsed
    })

    // aria-label tracks the current tag set so SR announcement matches
    // what's on screen. Updated reactively whenever tagList changes.
    const stopAriaLabel = effect(() => {
      const list = state.tagList
      const phrase =
        list.length === 0
          ? 'Tag input, no tags'
          : `Tags, ${list.length} selected: ${list.join(', ')}`
      host.setAttribute('aria-label', phrase)
    })

    const cap = (): number => {
      const n = state.maxTags
      return Number.isFinite(n) && n > 0 ? n : Infinity
    }

    const addTag = (raw: string) => {
      if (state.disabled) return
      const v = raw.trim()
      if (!v) return
      const cur = state.tagList
      if (cur.includes(v)) {
        state.entryValue = ''
        return
      }
      if (cur.length >= cap()) return
      writeBack([...cur, v])
      state.entryValue = ''
    }

    const removeAt = (idx: number) => {
      if (state.disabled) return
      const cur = state.tagList
      if (idx < 0 || idx >= cur.length) return
      writeBack(cur.filter((_: string, i: number) => i !== idx))
    }

    const inputEl = state.inputEl as HTMLInputElement | null
    const onKeydown = (e: KeyboardEvent) => {
      if (!inputEl) return
      if (e.key === 'Enter') {
        e.preventDefault()
        addTag(inputEl.value)
      } else if (e.key === 'Backspace' && inputEl.value.length === 0) {
        const cur = state.tagList
        if (cur.length === 0) return
        e.preventDefault()
        removeAt(cur.length - 1)
      } else if (e.key === ',') {
        // Comma also commits a tag, matching CSV semantics of `value`.
        e.preventDefault()
        addTag(inputEl.value)
      }
    }
    inputEl?.addEventListener('keydown', onKeydown)

    // Delegate `ship-remove` from any child <sui-chip removable> back to
    // the controller. Chips set data-idx so we know which tag fired.
    const onShadowRemove = (e: Event) => {
      const target = e.target as HTMLElement | null
      const chip = target?.closest?.('sui-chip[data-idx]') as HTMLElement | null
      if (!chip) return
      const idx = Number(chip.dataset.idx)
      if (Number.isInteger(idx)) removeAt(idx)
    }
    host.shadowRoot?.addEventListener('ship-remove', onShadowRemove)

    host.expose!({
      focus: () => (state.inputEl as HTMLInputElement | null)?.focus(),
      blur: () => (state.inputEl as HTMLInputElement | null)?.blur(),
      clear: () => writeBack([]),
      addTag: (label: string) => addTag(label),
    })

    return () => {
      stopAriaLabel()
      stopValueSync()
      inputEl?.removeEventListener('keydown', onKeydown)
      host.shadowRoot?.removeEventListener('ship-remove', onShadowRemove)
    }
  },
})
