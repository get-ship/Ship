import { controller, signal, derived, live } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ ship, route }) => {
    const skillId = derived(() => (route as any)().params.id)
    const q = signal('')
    const kind = signal('all')
    const tag = signal('')
    const selectedFile = signal('')
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any
    const rail = live(() => s.skills.summaries({ q: q(), kind: kind(), tag: tag() }))
    const skill = live(() => s.skills.detail({ id: skillId() }))
    const fileContent = live(() => s.skills.file({ id: skillId(), path: selectedFile() }))
    return { skillId, q, kind, tag, selectedFile, rail, skill, fileContent }
  },
  on: {
    selectFile: ({ state, navigate }, f: string) => {
      state.selectedFile = f
      navigate('/skills/' + (state.skillId as string) + '?file=' + encodeURIComponent(f))
    },
  },
  mount: ({ state, route }) => {
    const fileParam = (route as any)().query?.file
    if (fileParam) state.selectedFile = fileParam
  },
})
