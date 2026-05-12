import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ route }) => {
    const skillId = derived(() => (route as any)().params.id)
    const q = signal('')
    const kind = signal('all')
    const tag = signal('')
    const selectedFile = signal('')
    return { skillId, q, kind, tag, selectedFile }
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
