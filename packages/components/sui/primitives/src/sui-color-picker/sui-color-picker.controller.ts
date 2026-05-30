import { controller, derived, effect, ref, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-color-picker.sui'

interface Swatch { hex: string; label: string }

const SWATCHES: Swatch[] = [
  { hex: '#000000', label: 'Black' },
  { hex: '#ffffff', label: 'White' },
  { hex: '#6b7280', label: 'Gray' },
  { hex: '#ef4444', label: 'Red' },
  { hex: '#f97316', label: 'Orange' },
  { hex: '#eab308', label: 'Yellow' },
  { hex: '#22c55e', label: 'Green' },
  { hex: '#10b981', label: 'Emerald' },
  { hex: '#06b6d4', label: 'Cyan' },
  { hex: '#3b82f6', label: 'Blue' },
  { hex: '#8b5cf6', label: 'Violet' },
  { hex: '#ec4899', label: 'Pink' },
]

let idSeq = 0

const isHex = (s: string): boolean => /^#[0-9a-f]{6}$/i.test(s)

const normHex = (s: string): string => {
  const t = s.trim().toLowerCase()
  if (/^#[0-9a-f]{6}$/.test(t)) return t
  if (/^[0-9a-f]{6}$/.test(t)) return `#${t}`
  if (/^#[0-9a-f]{3}$/.test(t)) {
    return `#${t[1]}${t[1]}${t[2]}${t[2]}${t[3]}${t[3]}`
  }
  return ''
}

const hexToRgb = (hex: string): { r: number; g: number; b: number } | null => {
  if (!isHex(hex)) return null
  return {
    r: parseInt(hex.slice(1, 3), 16),
    g: parseInt(hex.slice(3, 5), 16),
    b: parseInt(hex.slice(5, 7), 16),
  }
}

const rgbToHex = (r: number, g: number, b: number): string => {
  const to2 = (n: number) => {
    const v = Math.max(0, Math.min(255, Math.round(n)))
    return v.toString(16).padStart(2, '0')
  }
  return `#${to2(r)}${to2(g)}${to2(b)}`
}

const rgbToHsl = (r: number, g: number, b: number): { h: number; s: number; l: number } => {
  const rn = r / 255, gn = g / 255, bn = b / 255
  const max = Math.max(rn, gn, bn)
  const min = Math.min(rn, gn, bn)
  const l = (max + min) / 2
  let h = 0, s = 0
  if (max !== min) {
    const d = max - min
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
    if (max === rn) h = ((gn - bn) / d + (gn < bn ? 6 : 0))
    else if (max === gn) h = (bn - rn) / d + 2
    else h = (rn - gn) / d + 4
    h *= 60
  }
  return { h: Math.round(h), s: Math.round(s * 100), l: Math.round(l * 100) }
}

const hslToHex = (h: number, s: number, l: number): string => {
  const sn = s / 100, ln = l / 100
  const c = (1 - Math.abs(2 * ln - 1)) * sn
  const hp = ((h % 360) + 360) % 360 / 60
  const x = c * (1 - Math.abs((hp % 2) - 1))
  let r1 = 0, g1 = 0, b1 = 0
  if (hp < 1) { r1 = c; g1 = x; b1 = 0 }
  else if (hp < 2) { r1 = x; g1 = c; b1 = 0 }
  else if (hp < 3) { r1 = 0; g1 = c; b1 = x }
  else if (hp < 4) { r1 = 0; g1 = x; b1 = c }
  else if (hp < 5) { r1 = x; g1 = 0; b1 = c }
  else { r1 = c; g1 = 0; b1 = x }
  const m = ln - c / 2
  return rgbToHex((r1 + m) * 255, (g1 + m) * 255, (b1 + m) * 255)
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const open = signal(false)
    const hue = signal(0)
    const sat = signal(0)
    const light = signal(0)
    const hueText = signal('0')
    const satText = signal('0')
    const lightText = signal('0')
    const hexDraft = signal('')
    const inputEl = ref<HTMLInputElement>(null)
    const panelEl = ref<HTMLDivElement>(null)
    const hexInputEl = ref<HTMLInputElement>(null)
    const id = ++idSeq
    const hueId = signal(`sui-color-picker-h-${id}`)
    const satId = signal(`sui-color-picker-s-${id}`)
    const lightId = signal(`sui-color-picker-l-${id}`)
    const swatches = derived<Swatch[]>(() => SWATCHES)
    return {
      open, hue, sat, light, hueText, satText, lightText, hexDraft,
      inputEl, panelEl, hexInputEl, hueId, satId, lightId, swatches,
    }
  },

  mount(state, host) {
    let suppress = false

    const emit = (hex: string) => {
      host.dispatchEvent(
        new CustomEvent('sui-change', {
          detail: { value: hex },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const syncFromHex = (hex: string) => {
      const rgb = hexToRgb(hex)
      if (!rgb) return
      const hsl = rgbToHsl(rgb.r, rgb.g, rgb.b)
      suppress = true
      state.hue = hsl.h
      state.sat = hsl.s
      state.light = hsl.l
      state.hueText = String(hsl.h)
      state.satText = String(hsl.s)
      state.lightText = String(hsl.l)
      state.hexDraft = hex
      suppress = false
    }

    const commitHex = (raw: string) => {
      const norm = normHex(raw)
      if (!norm) return false
      state.value = norm
      syncFromHex(norm)
      emit(norm)
      return true
    }

    // External value → internal HSL.
    const stopValueSync = effect(() => {
      const v = state.value
      if (suppress) return
      if (isHex(v)) syncFromHex(v)
    })

    // Initial seed.
    if (isHex(state.value)) syncFromHex(state.value)

    // Slider edits → value.
    const sliderEffect = effect(() => {
      const h = Number(state.hueText)
      const s = Number(state.satText)
      const l = Number(state.lightText)
      if (suppress) return
      if (!Number.isFinite(h) || !Number.isFinite(s) || !Number.isFinite(l)) return
      const hex = hslToHex(h, s, l)
      state.hue = Math.round(h)
      state.sat = Math.round(s)
      state.light = Math.round(l)
      if (hex !== state.value.toLowerCase()) {
        suppress = true
        state.value = hex
        state.hexDraft = hex
        suppress = false
        emit(hex)
      }
    })

    const inputEl = state.inputEl as HTMLInputElement | null
    if (!inputEl) return

    const openPanel = () => {
      if (state.disabled || state.open) return
      state.open = true
    }
    const closePanel = () => {
      if (!state.open) return
      state.open = false
    }

    const onInputClick = () => openPanel()
    const onInputChange = () => {
      // User typed a hex in the trigger input.
      const v = inputEl.value.trim()
      if (v === '') return
      if (!commitHex(v)) inputEl.value = state.value
    }
    const onInputKeydown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && state.open) {
        e.preventDefault()
        closePanel()
      } else if (e.key === 'ArrowDown' && !state.open) {
        e.preventDefault()
        openPanel()
      }
    }
    inputEl.addEventListener('click', onInputClick)
    inputEl.addEventListener('change', onInputChange)
    inputEl.addEventListener('keydown', onInputKeydown)

    const onShadowClick = (e: Event) => {
      const t = e.target as HTMLElement | null
      const sw = t?.closest?.('button.swatch[data-hex]') as HTMLElement | null
      if (sw) {
        const hex = (sw.dataset.hex ?? '').toLowerCase()
        if (isHex(hex)) commitHex(hex)
      }
    }
    const onHexInputChange = (e: Event) => {
      const t = e.target as HTMLInputElement | null
      if (!t || !t.matches('.hex-row input')) return
      if (!commitHex(t.value)) t.value = state.value
    }

    const onDocPointer = (e: Event) => {
      if (!state.open) return
      const path = e.composedPath()
      if (path.includes(host)) return
      closePanel()
    }

    host.shadowRoot?.addEventListener('click', onShadowClick)
    host.shadowRoot?.addEventListener('change', onHexInputChange)
    document.addEventListener('pointerdown', onDocPointer)

    host.expose!({
      focus: () => inputEl.focus(),
      blur: () => inputEl.blur(),
      open: openPanel,
      close: closePanel,
    })

    return () => {
      stopValueSync()
      sliderEffect()
      inputEl.removeEventListener('click', onInputClick)
      inputEl.removeEventListener('change', onInputChange)
      inputEl.removeEventListener('keydown', onInputKeydown)
      host.shadowRoot?.removeEventListener('click', onShadowClick)
      host.shadowRoot?.removeEventListener('change', onHexInputChange)
      document.removeEventListener('pointerdown', onDocPointer)
    }
  },
})
