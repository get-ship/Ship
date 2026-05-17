// Skills view controller — IDE shell that handles both `/skills` (no id) and
// `/skills/:id` (detail). The shell stays mounted across navigation between
// list/detail and sibling detail rows because main.ts treats `skills` as a
// SAME_SHELL_VIEWS root: only `route.params.id()` flips, the controller
// reacts in place, no payload re-fetch, no DOM flash.
//
// liveExpr is used directly (no SDK client in ctx for /sui-hosted views).
// Editable surfaces:
//   PUT /api/v1/skills/{id}/body          — markdown body
//   PUT /api/v1/skills/{id}/files/{path}  — companion files
//   PUT /api/v1/skills/{id}/frontmatter   — description + tags + name

import { controller, signal, derived, live, effect } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { Scope } from './page.sui'

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
type Banner = { ok: true; msg: string } | { ok: false; msg: string } | null

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
    case 'md':   return { icon: 'file-text', tone: name === 'SKILL.md' ? 'primary' : 'success', kind }
    case 'json': return { icon: 'file-code', tone: 'warning', kind }
    case 'yaml':
    case 'sh':
    case 'ts':   return { icon: 'file-code', tone: 'info', kind }
    case 'txt':  return { icon: 'file-text', tone: 'muted', kind }
    default:     return { icon: 'file', tone: 'muted', kind }
  }
}

// Inverse of the daemon's html_escape — `& < > " '` only.
function decodeEntities(s: string): string {
  return s
    .replace(/&#x27;/g, "'")
    .replace(/&quot;/g, '"')
    .replace(/&gt;/g, '>')
    .replace(/&lt;/g, '<')
    .replace(/&amp;/g, '&')
}

function isEditable(path: string, ext: string): boolean {
  if (!path) return true
  return ext === 'md'
}

function splitTags(s: string): string[] {
  return s.split(',').map((t) => t.trim()).filter(Boolean)
}

function readSkillIdFromPath(path: string): string {
  const m = path.match(/^\/skills\/([^/?#]+)/)
  return m ? decodeURIComponent(m[1]) : ''
}

function readFileFromSearch(search: string): string {
  if (!search) return ''
  const params = new URLSearchParams(search.startsWith('?') ? search.slice(1) : search)
  return params.get('file') ?? ''
}

export default controller<Scope>({
  // The shell's same-shell routing leaves this controller mounted across
  // `/skills` ⇄ `/skills/:id` navigation; the host route signals are
  // shell-bundle instances and won't fire view-bundle deriveds across
  // module boundaries, so we maintain our own URL signal and update it
  // on every popstate (shell dispatches a synthetic popstate after each
  // same-shell pushState).
  scope: () => {
    const pathSig = signal(typeof location !== 'undefined' ? location.pathname : '')
    const searchSig = signal(typeof location !== 'undefined' ? location.search : '')
    // `effect` is auto-disposed when the view's scope tears down — so we
    // can attach the popstate listener inside an effect's return cleanup
    // and rely on the runtime to remove it on view unmount.
    effect(() => {
      if (typeof window === 'undefined') return
      const onPop = () => {
        pathSig.set(location.pathname)
        searchSig.set(location.search)
      }
      window.addEventListener('popstate', onPop)
      window.addEventListener('sui:navigate', onPop)
      return () => {
        window.removeEventListener('popstate', onPop)
        window.removeEventListener('sui:navigate', onPop)
      }
    })
    const skillId = derived(() => readSkillIdFromPath(pathSig()))
    const selectedFile = derived(() => readFileFromSearch(searchSig()))
    const q = signal('')
    const kind = signal('all')
    const tag = signal('')

    // Body editor state
    const draft = signal('')
    const baseline = signal('')
    const saving = signal(false)
    const saveBanner = signal<Banner>(null)

    // Frontmatter editor state
    const descDraft = signal('')
    const descBaseline = signal('')
    const tagsDraft = signal('')
    const tagsBaseline = signal('')
    const savingMeta = signal(false)
    const metaBanner = signal<Banner>(null)

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

    // Seed body draft when selection or detail changes.
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

    // Seed frontmatter drafts whenever the loaded skill changes.
    effect(() => {
      const sk = skill()
      if (sk.kind !== 'ok' || !sk.value) return
      const desc = sk.value.description ?? ''
      const tags = (sk.value.tags ?? []).join(', ')
      descDraft.set(desc)
      descBaseline.set(desc)
      tagsDraft.set(tags)
      tagsBaseline.set(tags)
      metaBanner.set(null)
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
    const selectedFileName = derived(() => {
      const path = selectedFile()
      if (!path) return ''
      const idx = path.lastIndexOf('/')
      return idx >= 0 ? path.slice(idx + 1) : path
    })
    const activeIsEditable = derived(() => {
      const path = selectedFile()
      if (!path) return true
      const fc = fileContent()
      if (fc.kind === 'ok' && fc.value) return isEditable(path, fc.value.ext)
      return isEditable(path, fileKind(path))
    })
    const tagsList = derived<string[]>(() => splitTags(tagsDraft()))
    const authorsText = derived<string>(() => {
      const sk = skill()
      if (sk.kind !== 'ok' || !sk.value) return ''
      return (sk.value.authors ?? []).join(', ')
    })
    // Stable view of the loaded skill that survives mid-flight churn while
    // `live()` re-subscribes between sibling skill rows. Template guards
    // sometimes re-evaluate dependent bindings before tearing down the old
    // branch — `sk()` returns null instead of throwing on `.name` etc.
    const sk = derived<SkillDetail | null>(() => {
      const s = skill()
      return s.kind === 'ok' && s.value ? s.value : null
    })
    const skName = derived(() => sk()?.name ?? '')
    const skId = derived(() => sk()?.id ?? '')
    const skKind = derived(() => sk()?.kind ?? '')
    const skSource = derived(() => sk()?.source ?? '')
    const skBody = derived(() => sk()?.body ?? '')
    const skVariables = derived<string[]>(() => sk()?.variables ?? [])
    const skFiles = derived<string[]>(() => sk()?.files ?? [])
    const skAuthorsLen = derived(() => (sk()?.authors ?? []).length)
    const metaDirty = derived(() =>
      descDraft() !== descBaseline() || tagsDraft() !== tagsBaseline(),
    )

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

    async function saveMeta(): Promise<void> {
      const id = skillId()
      const sk = skill()
      if (!id || savingMeta() || sk.kind !== 'ok' || !sk.value) return
      savingMeta.set(true)
      metaBanner.set(null)
      // Replace-style endpoint — must preserve name + any unknown keys.
      // We only edit description + tags; rebuild the frontmatter from the
      // current detail plus our drafts.
      const fm: Record<string, unknown> = {
        name: sk.value.name,
        description: descDraft(),
      }
      const tags = splitTags(tagsDraft())
      if (tags.length > 0) fm.tags = tags
      if ((sk.value.authors ?? []).length > 0) fm.authors = sk.value.authors
      try {
        const resp = await fetch(`/api/v1/skills/${encodeURIComponent(id)}/frontmatter`, {
          method: 'PUT',
          credentials: 'same-origin',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify(fm),
        })
        const text = await resp.text().catch(() => '')
        if (!resp.ok) {
          metaBanner.set({ ok: false, msg: text || `Save failed (${resp.status})` })
          return
        }
        descBaseline.set(descDraft())
        tagsBaseline.set(tagsDraft())
        metaBanner.set({ ok: true, msg: 'Metadata saved.' })
      } catch (e) {
        metaBanner.set({ ok: false, msg: (e as Error).message ?? 'Save failed.' })
      } finally {
        savingMeta.set(false)
      }
    }

    return {
      skillId, q, kind, tag, selectedFile,
      rail, skill, fileContent, fileMetas,
      draft, dirty, saving, saveBanner, activeIsEditable, selectedFileName,
      descDraft, tagsDraft, tagsList, authorsText, savingMeta, metaDirty, metaBanner,
      sk, skName, skId, skKind, skSource, skBody, skVariables, skFiles, skAuthorsLen,
      onEditorInput, save, saveMeta,
    }
  },
})
