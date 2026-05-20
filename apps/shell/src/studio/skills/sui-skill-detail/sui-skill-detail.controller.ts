import { controller, signal, live, chrome } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { Scope } from './sui-skill-detail.sui'

interface SkillDetail {
  id: string
  name: string
  kind?: string
  source?: string
  description?: string
  body?: string
  tags?: string[]
  authors?: string[]
  variables?: string[]
  files?: string[]
  used_by?: string[]
}

const TILE_COLORS = ['#d97757', '#c98a2c', '#b94a4a', '#7a5cd6', '#4376d6', '#2a8a8a', '#4f9e54', '#c25a8e', '#5b6878']

function tileColor(id: string): string {
  return TILE_COLORS[(id.codePointAt(0) ?? 0) % TILE_COLORS.length]
}

function tileChar(name: string): string {
  return (name[0] ?? '?').toUpperCase()
}

function sourceLabel(src: string): string {
  if (src === 'PROJECT SKILLS') return 'Project · .ship/'
  if (src === 'USER SKILLS') return 'User · ~/.ship/'
  return src
}

type InvokeStatus = 'idle' | 'loading' | 'ok' | 'error'

export interface InvokeVar { id: number; key: string; value: string }

interface InvokeResult {
  run_id: string
  skill_id: string
  session_id: string
  workspace_id: string | null
  mode: string
  reused_session: boolean
}

export interface RenderedSkill {
  id: string
  name: string
  kind: string
  sourceStr: string
  description: string
  body: string
  tags: string[]
  authors: string[]
  authorsText: string
  variables: string[]
  files: string[]
  usedBy: string[]
  usedByCount: number
  color: string
  char: string
}

function renderSkill(sk: SkillDetail): RenderedSkill {
  const authors = sk.authors ?? []
  const usedBy = sk.used_by ?? []
  return {
    id: sk.id,
    name: sk.name,
    kind: sk.kind ?? '',
    sourceStr: sourceLabel(sk.source ?? ''),
    description: sk.description ?? '',
    body: sk.body ?? '',
    tags: sk.tags ?? [],
    authors,
    authorsText: authors.join(', '),
    variables: sk.variables ?? [],
    files: sk.files ?? [],
    usedBy,
    usedByCount: usedBy.length,
    color: tileColor(sk.id),
    char: tileChar(sk.name),
  }
}

let _nextId = 0
const nextId = () => ++_nextId

function parseSkillId(): string {
  const parts = typeof window !== 'undefined'
    ? window.location.pathname.split('/').filter(Boolean)
    : []
  return parts.length >= 3 && parts[0] === 'studio' && parts[1] === 'skills'
    ? parts[2]
    : ''
}

export default controller<Scope>({
  scope: () => {
    const skillId = parseSkillId
    const detail = live(() =>
      liveExpr<SkillDetail>(
        `ship.skills.detail(${JSON.stringify({ id: skillId() || '__none__' })})`,
      ),
    )

    const showInvoke = signal(false)
    const invokeVars = signal<InvokeVar[]>([{ id: nextId(), key: '', value: '' }])
    const invokeStatus = signal<InvokeStatus>('idle')
    const invokeResult = signal<InvokeResult | null>(null)
    const invokeError = signal('')

    const toggleInvoke = () => showInvoke.set(!showInvoke())
    const back = () => chrome.navigate('/studio/skills')
    const addVar = () => invokeVars.set([...invokeVars(), { id: nextId(), key: '', value: '' }])
    const removeVar = (id: number) => invokeVars.set(invokeVars().filter((kv) => kv.id !== id))
    const setVarKey = (id: number, e: Event) => {
      const v = (e.target as HTMLInputElement).value
      invokeVars.set(invokeVars().map((kv) => (kv.id === id ? { ...kv, key: v } : kv)))
    }
    const setVarVal = (id: number, e: Event) => {
      const v = (e.target as HTMLInputElement).value
      invokeVars.set(invokeVars().map((kv) => (kv.id === id ? { ...kv, value: v } : kv)))
    }

    const invoke = async () => {
      const id = skillId()
      if (!id || invokeStatus() === 'loading') return
      const vars: Record<string, string> = {}
      for (const { key, value } of invokeVars()) {
        if (key.trim()) vars[key.trim()] = value
      }
      invokeStatus.set('loading')
      invokeResult.set(null)
      invokeError.set('')
      try {
        const resp = await fetch(`/api/v1/skills/${encodeURIComponent(id)}/invoke`, {
          method: 'POST',
          credentials: 'same-origin',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ vars }),
        })
        const data = await resp.json().catch(() => null)
        if (!resp.ok) {
          invokeStatus.set('error')
          invokeError.set((data as { error?: string })?.error ?? `Error ${resp.status}`)
          return
        }
        invokeStatus.set('ok')
        invokeResult.set(data as InvokeResult)
      } catch (e) {
        invokeStatus.set('error')
        invokeError.set((e as Error).message ?? 'Invoke failed')
      }
    }

    const resetInvoke = () => {
      invokeStatus.set('idle')
      invokeResult.set(null)
      invokeError.set('')
    }

    return {
      detail, showInvoke, invokeVars, invokeStatus, invokeResult, invokeError,
      toggleInvoke, back, addVar, removeVar, setVarKey, setVarVal, invoke, resetInvoke,
    }
  },

  derived: {
    skill: ({ detail }) => {
      const st = detail()
      if (st.kind !== 'ok' || !st.value) return null as RenderedSkill | null
      return renderSkill(st.value as SkillDetail)
    },
  },
})
