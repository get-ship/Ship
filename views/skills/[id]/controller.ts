// Skill detail controller — IDE shell for `/skills/:id`.
//
// Wires three live channels (rail/skill/file) directly through `liveExpr`
// rather than through `ctx.ship.skills.*` because the view-host
// (`/sui/<name>`) mounts without an SDK client and `ship.skills.file`
// is not part of the typed SDK surface (only summaries/detail/list/invoke
// are — `file` is registry-only).
//
// The TipTap-backed `<sui-markdown-editor>` is wired to the `draft`
// signal. We seed `draft` from the loaded file content (or SKILL.md
// body when no file is selected) and propagate user edits via the
// editor's `sui-change` event. `save()` posts to
// PUT /api/v1/skills/{id}/body for SKILL.md, or
// PUT /api/v1/skills/{id}/files/{path} for any other file.
//
// File content from `ship.skills.file` is HTML-escaped at the service
// boundary (because the user-view env runs AutoEscape::None for
// Datastar attr compatibility) — we decode it before handing it to the
// editor so the editor doesn't display literal `&amp;` / `&lt;`.

import { controller, signal, derived, live, effect, defineRoutes } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { Scope } from './page.sui'

defineRoutes([{ pattern: '/skills/:id' }])

type FileKind = 'md' | 'json' | 'yaml' | 'sh' | 'ts' | 'txt' | 'unknown'
type FileMeta = { icon: string; tone: string; kind: FileKind }

interface SkillSummary {
  id: string
  name: string
  description?: string
  has_vars?: boolean
  has_refs?: boolean
  has_evals?: boolean
  files?: string[]
}
interface SkillGroup { source: string; skills: SkillSummary[] }
interface RailValue {
  groups: SkillGroup[]
  kind_counts: Array<{ id: string; label: string; count: number; selected: boolean }>
}
interface SkillDetail {
  id: string
  name: string
  kind: string
  source: string
  description?: string
  body?: string
  tags?: string[]
  authors?: string[]
  variables?: string[]
  files?: string[]
}
interface SkillFile {
  path: string
  name: string
  ext: string
  content: string
}
type SaveBanner = { ok: true; msg: string } | { ok: false; msg: string } | null

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

// Inverse of the daemon's `html_escape` helper. The service emits only
// `& < > " '` entities, so a focused decode covers it.
function decodeEntities(s: string): string {
  return s
    .replace(/&#x27;/g, "'")
    .replace(/&quot;/g, '"')
    .replace(/&gt;/g, '>')
    .replace(/&lt;/g, '<')
    .replace(/&amp;/g, '&')
}

function fileHref(id: string, file: string): string {
  return `/skills/${encodeURIComponent(id)}?file=${encodeURIComponent(file)}`
}

function isEditable(path: string, ext: string): boolean {
  if (!path) return true
  return ext === 'md'
}

export default controller<Scope>({
  scope: ({ route }) => {
    // skillId mirrors the URL; live channels close + reopen when it
    // changes. `selectedFile` mirrors `?file=` so deep-linking and
    // back/forward both preserve the open file.
    const skillId = derived(() => (route?.params?.id?.() as string | undefined) ?? '')
    const selectedFile = derived(() => (route?.query?.file?.() as string | undefined) ?? '')
    const q = signal('')
    const kind = signal('all')
    const tag = signal('')
    const draft = signal('')
    const baseline = signal('')
    const saving = signal(false)
    const saveBanner = signal<SaveBanner>(null)

    const rail = live(() => liveExpr<RailValue>(
      `ship.skills.summaries(${JSON.stringify({ q: q(), kind: kind(), tag: tag() })})`,
    ))
    const skill = live(() => liveExpr<SkillDetail>(
      `ship.skills.detail(${JSON.stringify({ id: skillId() || '__none__' })})`,
    ))
    const fileContent = live(() => liveExpr<SkillFile>(
      `ship.skills.file(${JSON.stringify({
        id: skillId() || '__none__',
        path: selectedFile() || '__none__',
      })})`,
    ))

    // Seed the editor draft from whatever content is currently loaded.
    // Re-runs when the selection changes or the live data lands.
    effect(() => {
      const path = selectedFile()
      if (path) {
        const fc = fileContent()
        if (fc.kind === 'ok' && fc.value && fc.value.path === path) {
          const decoded = decodeEntities(fc.value.content)
          draft.set(decoded)
          baseline.set(decoded)
          saveBanner.set(null)
        }
      } else {
        const sk = skill()
        if (sk.kind === 'ok' && sk.value) {
          const body = sk.value.body ?? ''
          draft.set(body)
          baseline.set(body)
          saveBanner.set(null)
        }
      }
    })

    const fileMetas = derived<Record<string, FileMeta>>(() => {
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

    const dirty = derived(() => draft() !== baseline())
    const activeIsEditable = derived(() => {
      const path = selectedFile()
      if (!path) return true
      const fc = fileContent()
      if (fc.kind === 'ok' && fc.value) return isEditable(path, fc.value.ext)
      return isEditable(path, fileKind(path))
    })

    function onEditorInput(evt: Event): void {
      const detail = (evt as CustomEvent<{ value?: string }>).detail
      if (detail && typeof detail.value === 'string') {
        draft.set(detail.value)
      } else {
        const el = evt.target as HTMLElement & { value?: string }
        if (typeof el.value === 'string') draft.set(el.value)
      }
    }

    async function save(): Promise<void> {
      const id = skillId()
      if (!id || saving()) return
      const path = selectedFile()
      const body = draft()
      saving.set(true)
      saveBanner.set(null)
      const url = path
        ? `/api/v1/skills/${encodeURIComponent(id)}/files/${path.split('/').map(encodeURIComponent).join('/')}`
        : `/api/v1/skills/${encodeURIComponent(id)}/body`
      const payload = path ? { content: body } : { body }
      try {
        const resp = await fetch(url, {
          method: 'PUT',
          credentials: 'same-origin',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify(payload),
        })
        const text = await resp.text().catch(() => '')
        if (!resp.ok) {
          saveBanner.set({ ok: false, msg: text || `Save failed (${resp.status})` })
          return
        }
        baseline.set(body)
        saveBanner.set({ ok: true, msg: 'Saved.' })
      } catch (e) {
        saveBanner.set({ ok: false, msg: (e as Error).message ?? 'Save failed.' })
      } finally {
        saving.set(false)
      }
    }

    return {
      skillId, q, kind, tag, selectedFile,
      rail, skill, fileContent, fileMetas, fileHref,
      draft, dirty, saving, saveBanner, activeIsEditable,
      onEditorInput, save,
    }
  },
})
