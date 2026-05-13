import { controller, signal, derived, live, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ route, ship }) => {
    const workspaceId = derived(() => (route as any)().query?.workspace_id || '')
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const chatState = live(() => (ship as any).chat.state({ workspace_id: workspaceId() }))
    const draft = signal('')
    const sending = signal(false)
    const workingTitle = signal('')
    const annotations = signal<{ selector?: string; tag?: string; id?: string; preview?: string }[]>([])
    return { workspaceId, chatState, draft, sending, workingTitle, annotations }
  },
  on: {
    submitMessage: async (s, e: Event) => {
      e.preventDefault()
      const d = s.draft as string
      if (!d.trim() && !(s.annotations as unknown[]).length) return
      s.sending = true
      const ann = s.annotations as { selector?: string; tag?: string; preview?: string }[]
      const annBlock = ann.length
        ? '\n\nContext:\n' + ann.map(a => '- ' + (a.selector || a.tag || 'ref') + (a.preview ? ': ' + a.preview : '')).join('\n')
        : ''
      await fetch('/api/v1/chat/send', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ text: d + annBlock, workspace_id: s.workspaceId || null }),
      })
      s.draft = ''
      s.annotations = []
      s.sending = false
    },
    removeAnnotation: (s, a: object) => {
      s.annotations = (s.annotations as object[]).filter(x => x !== a)
    },
  },
  mount: ({ state }) => {
    const onSubmitted = (e: Event) => {
      state.workingTitle = (e as CustomEvent).detail?.name || ''
    }
    const onDone = () => { state.workingTitle = '' }
    const onAnnotate = (e: Event) => {
      state.annotations = [...(state.annotations as object[]), (e as CustomEvent).detail]
    }
    window.addEventListener('ship:create:submitted', onSubmitted)
    window.addEventListener('ship:create:done', onDone)
    window.addEventListener('ship:chat:annotate', onAnnotate)
    ;(state as Record<string, unknown>)._cleanup = () => {
      window.removeEventListener('ship:create:submitted', onSubmitted)
      window.removeEventListener('ship:create:done', onDone)
      window.removeEventListener('ship:chat:annotate', onAnnotate)
    }
  },
  unmount: ({ state }) => {
    const cleanup = (state as Record<string, unknown>)._cleanup
    if (typeof cleanup === 'function') cleanup()
  },
})
