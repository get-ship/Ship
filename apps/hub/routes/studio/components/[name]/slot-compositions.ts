// Last-resort default-slot label for a component with no co-located examples.
// Real showcase content lives in each component's `<tag>.examples.ts` (surfaced
// via the CEM) — there is no central composition map anymore.
//
// Atomic-component default slot text: the humanized tag, so each component
// shows its own name (sui-button → "Button") instead of a blank box.
export function slotLabel(tag: string): string {
  const base = tag.replace(/^sui-/, '').replace(/-/g, ' ').trim()
  return base ? base.charAt(0).toUpperCase() + base.slice(1) : 'Preview'
}
