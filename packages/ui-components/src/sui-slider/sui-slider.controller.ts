import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-slider.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    // Light-DOM hidden input for form participation.
    const hiddenInput = document.createElement('input')
    hiddenInput.type = 'hidden'
    host.appendChild(hiddenInput)

    const clamp = (v: number): number => {
      const step = scope.step() as number
      const min = scope.min() as number
      const max = scope.max() as number
      const stepped = Math.round((v - min) / step) * step + min
      return Math.max(min, Math.min(max, stepped))
    }

    const setValue = (v: number) => {
      const clamped = clamp(v)
      if (clamped === (scope.value() as number)) return
      scope.value.set(clamped)
      hiddenInput.name = scope.name() as string
      hiddenInput.value = String(clamped)
      host.dispatchEvent(new CustomEvent('sui-change', {
        detail: { value: clamped },
        bubbles: true,
        composed: true,
      }))
    }

    const valueFromPointer = (clientX: number): number => {
      const track = scope.trackEl() as HTMLElement | null
      if (!track) return scope.value() as number
      const rect = track.getBoundingClientRect()
      const pct = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width))
      return (scope.min() as number) + pct * ((scope.max() as number) - (scope.min() as number))
    }

    // Sync fill + thumb position and hidden input on value/range changes.
    const stopPositionSync = effect(() => {
      const val = scope.value() as number
      const min = scope.min() as number
      const max = scope.max() as number
      const range = max - min || 1
      const pct = ((val - min) / range) * 100

      const fill = scope.fillEl() as HTMLElement | null
      const thumb = scope.thumbEl() as HTMLElement | null
      if (fill) fill.style.width = `${pct}%`
      if (thumb) thumb.style.left = `${pct}%`

      hiddenInput.name = scope.name() as string
      hiddenInput.value = String(val)
    })

    let dragging = false

    const onPointerDown = (e: PointerEvent) => {
      if (scope.disabled() as boolean) return
      e.preventDefault()
      dragging = true
      setValue(valueFromPointer(e.clientX))
      const thumb = scope.thumbEl() as HTMLElement | null
      thumb?.focus()

      const onMove = (ev: PointerEvent) => { if (dragging) setValue(valueFromPointer(ev.clientX)) }
      const onUp = () => {
        dragging = false
        document.removeEventListener('pointermove', onMove)
        document.removeEventListener('pointerup', onUp)
      }
      document.addEventListener('pointermove', onMove)
      document.addEventListener('pointerup', onUp)
    }

    const onKeydown = (e: KeyboardEvent) => {
      if (scope.disabled() as boolean) return
      const step = scope.step() as number
      let handled = true
      switch (e.key) {
        case 'ArrowRight':
        case 'ArrowUp':
          setValue((scope.value() as number) + step)
          break
        case 'ArrowLeft':
        case 'ArrowDown':
          setValue((scope.value() as number) - step)
          break
        case 'Home':
          setValue(scope.min() as number)
          break
        case 'End':
          setValue(scope.max() as number)
          break
        case 'PageUp':
          setValue((scope.value() as number) + step * 10)
          break
        case 'PageDown':
          setValue((scope.value() as number) - step * 10)
          break
        default:
          handled = false
      }
      if (handled) e.preventDefault()
    }

    const stopPointer = effect(() => {
      const container = scope.containerEl() as HTMLElement | null
      if (!container) return
      container.addEventListener('pointerdown', onPointerDown as EventListener)
      return () => container.removeEventListener('pointerdown', onPointerDown as EventListener)
    })

    const stopKeydown = effect(() => {
      const thumb = scope.thumbEl() as HTMLElement | null
      if (!thumb) return
      thumb.addEventListener('keydown', onKeydown)
      return () => thumb.removeEventListener('keydown', onKeydown)
    })

    return () => {
      stopPositionSync()
      stopPointer()
      stopKeydown()
      hiddenInput.remove()
    }
  },
})
