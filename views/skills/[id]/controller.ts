import { controller, signal, derived, live, defineRoutes } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

defineRoutes([{ pattern: '/skills/:id' }])

type FileKind = 'md' | 'json' | 'yaml' | 'sh' | 'ts' | 'txt' | 'unknown'
type FileMeta = { icon: string; tone: string; kind: FileKind }

function fileKind(name: string): FileKind {
  const lower = name.toLowerCase()
  if (lower.endsWith('.md')) return 'md'
  if (lower.endsWith('.json') || lower.endsWith('.jsonc')) return 'json'
  if (lower.endsWith('.yaml') || lower.endsWith('.yml')) return 'yaml'
  if (lower.endsWith('.sh')) return 'sh'
  if (lower.endsWith('.ts') || lower.endsWith('.tsx') || lower.endsWith('.js')) return 'ts'
  if (lower.endsWith('.txt')) return 'txt'
  return 'unknown'
}

function fileMeta(name: string): FileMeta {
  const kind = fileKind(name)
  switch (kind) {
    case 'md':
      return { icon: 'file-text', tone: name === 'SKILL.md' ? 'primary' : 'success', kind }
    case 'json':
      // file-json is not in the icon registry; file-code reads as "structured/code"
      return { icon: 'file-code', tone: 'warning', kind }
    case 'yaml':
    case 'sh':
    case 'ts':
      return { icon: 'file-code', tone: 'info', kind }
    case 'txt':
      return { icon: 'file-text', tone: 'muted', kind }
    default:
      return { icon: 'file', tone: 'muted', kind }
  }
}

export default controller<Scope>({
  scope: ({ ship, route }) => {
    const skillId = derived(() => route.params.id() ?? '')
    const q = signal('')
    const kind = signal('all')
    const tag = signal('')
    const selectedFile = signal('')
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any
    const rail = live(() => s.skills.summaries({ q: q(), kind: kind(), tag: tag() }))
    const skill = live(() => s.skills.detail({ id: skillId() }))
    const fileContent = live(() => s.skills.file({ id: skillId(), path: selectedFile() }))
    const fileMetas = derived(() => {
      const sId = skillId()
      const r = rail()
      const map: Record<string, FileMeta> = {}
      if (r.kind !== 'ok' || !r.value) return map
      for (const g of r.value.groups || []) {
        for (const sk of g.skills) {
          if (sk.id === sId) {
            for (const f of sk.files || []) map[f] = fileMeta(f)
          }
        }
      }
      return map
    })
    return { skillId, q, kind, tag, selectedFile, rail, skill, fileContent, fileMetas }
  },
  on: {
    selectFile: ({ state, navigate }, f: string) => {
      state.selectedFile = f
      navigate('/skills/' + (state.skillId as string) + '?file=' + encodeURIComponent(f))
    },
  },
  mount: ({ state, route }) => {
    // route is undefined when mounted by the view-host harness (/sui/<name>).
    const fileParam = route?.query?.file?.()
    if (fileParam) state.selectedFile = fileParam
  },
})
