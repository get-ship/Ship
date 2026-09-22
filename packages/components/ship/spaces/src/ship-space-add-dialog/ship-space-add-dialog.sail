<!--
  ship-space-add-dialog.sail — the @ship/spaces ADD surface (SHELL side).
  Registers a directory as a space (SpaceService.Add). Submitting emits
  `ship-add` {path}; cancel emits `ship-cancel`. The consumer calls Add and,
  on failure, feeds the result back via `error` + `tone`. The surface never
  calls the daemon itself. Gated by `open`.
  Compiled as <ship-space-add-dialog>.
-->
<scope>
  props: {
    open = false
    value = ''
    error = ''
    tone: 'none' | 'warning' | 'destructive' = 'none'
  }
</scope>

<controller>
import { controller, onCleanup } from '@ship/sail-runtime'
import type { Scope } from './ship-space-add-dialog.sail'

export default controller((_ctx: Scope, host?: Element) => {
  if (!host) return {}

  const readPath = (): string => {
    // Production renders into the shadow root; the test harness renders
    // the template into the host's light DOM. `#p` is the directory field.
    const root = host.shadowRoot ?? host
    const input = root.querySelector<HTMLInputElement>('#p')
    return (input?.value || '').trim()
  }

  const submit = () => {
    const path = readPath()
    if (!path) return
    host.dispatchEvent(
      new CustomEvent('ship-add', { detail: { path }, bubbles: true, composed: true }),
    )
  }

  const cancel = () => {
    host.dispatchEvent(new CustomEvent('ship-cancel', { bubbles: true, composed: true }))
  }

  const onClick = (e: Event) => {
    for (const node of e.composedPath()) {
      if (!(node instanceof Element)) continue
      if (node.matches('[data-action="submit"]')) return submit()
      if (node.matches('[data-action="cancel"]')) return cancel()
    }
  }

  const onKey = (e: KeyboardEvent) => {
    if (!host.hasAttribute('open')) return
    if (e.key === 'Enter' && (e.target as Element)?.matches?.('#p')) {
      e.preventDefault()
      submit()
    } else if (e.key === 'Escape') {
      cancel()
    }
  }

  host.addEventListener('click', onClick)
  host.addEventListener('keydown', onKey)
  onCleanup(() => {
    host.removeEventListener('click', onClick)
    host.removeEventListener('keydown', onKey)
  })

  return {}
})
</controller>

<div role="dialog" aria-modal="true" aria-label="Register a space">
  <div>
    <span>Register a space</span>
    <span>Point Ship at an existing directory. It becomes an independent space — Ship ensures its <span>.ship/</span> on registration.</span>
  </div>

  <div>
    <label for="p">Directory</label>
    <input id="p" type="text" placeholder="/path/to/your/project" value={value()} spellcheck="false" autocomplete="off" />
  </div>

  <if {error()}>
    <div data-tone={tone()} role="alert">
      <icon name="circle-alert" size="sm"></icon>
      <span>{error()}</span>
    </div>
  </if>

  <div>
    <button type="button" data-action="cancel">Cancel</button>
    <button type="button" data-action="submit">Register</button>
  </div>
</div>
