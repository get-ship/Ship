// Skills view controller — IDE shell that handles both `/skills` (no id) and
// `/skills/:id` (detail). The shell stays mounted across navigation between
// list/detail and sibling detail rows because main.ts treats `skills` as a
// SAME_SHELL_VIEWS root: only `route.params.id()` flips, the controller
// reacts in place, no payload re-fetch, no DOM flash.
//
// liveExpr is used directly (no SDK client in ctx for /sui-hosted views).
// Editable surfaces (file-system source of truth):
//   PUT /api/v1/skills/{id}/body          — markdown body
//   PUT /api/v1/skills/{id}/files/{path}  — companion files
//   PUT /api/v1/skills/{id}/frontmatter   — description + tags + name + …
//
// Metadata panel sections (right rail), each `<sui-collapsible>`:
//   identity · description · tags · triggers · variables · references · evals
// Per-section open state persists in localStorage keyed by skill id.

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

const SECTION_IDS = ['identity', 'description', 'tags', 'triggers', 'variables', 'references', 'evals'] as const
type SectionId = typeof SECTION_IDS[number]
const DEFAULT_OPEN: Record<SectionId, boolean> = {
  identity: true, description: true,
  tags: false, triggers: false, variables: false, references: false, evals: false,
}
const STORAGE_PREFIX = 'ship.skills.metadata-open:'

function loadOpenState(id: string): Record<SectionId, boolean> {
  if (!id || typeof localStorage === 'undefined') return { ...DEFAULT_OPEN }
  try {
    const raw = localStorage.getItem(STORAGE_PREFIX + id)
    if (!raw) return { ...DEFAULT_OPEN }
    const parsed = JSON.parse(raw) as Partial<Record<SectionId, boolean>>
    const out = { ...DEFAULT_OPEN }
    for (const k of SECTION_IDS) if (typeof parsed[k] === 'boolean') out[k] = parsed[k]!
    return out
  } catch {
    return { ...DEFAULT_OPEN }
  }
}

function saveOpenState(id: string, state: Record<SectionId, boolean>): void {
  if (!id || typeof localStorage === 'undefined') return
  try {
    localStorage.setItem(STORAGE_PREFIX + id, JSON.stringify(state))
  } catch {
    // localStorage may throw (quota / private mode); silently ignore — state
    // simply reverts to defaults next session.
  }
}

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

function readSkillIdFromPath(path: string): string {
  // Tolerate both the shell-SPA shape (`/skills/<id>`) and the standalone
  // view-host shape (`/view-host/skills/<id>`) so the view renders end-to-end
  // in both contexts. The trailing `[^/?#]+` keeps the match scoped to a
  // single path segment so sub-routes (`/skills/<id>/edit`) still resolve to
  // the parent id.
  const m = path.match(/(?:^|\/)skills\/([^/?#]+)/)
  return m ? decodeURIComponent(m[1]) : ''
}

function readFileFromSearch(search: string): string {
  if (!search) return ''
  const params = new URLSearchParams(search.startsWith('?') ? search.slice(1) : search)
  return params.get('file') ?? ''
}

function sourceLabel(src: string): string {
  if (src === 'PROJECT SKILLS') return 'Project · .ship/'
  if (src === 'USER SKILLS') return 'User · ~/.ship/'
  return src
}

// Light heuristic: capture body text under TRIGGER/SKIP/WHEN lines so the
// Triggers section has *something* to render. Skills declare invocation cues
// in the markdown body, not as a frontmatter field — so we extract a hint.
function extractTriggerHints(body: string): string[] {
  if (!body) return []
  const lines = body.split(/\r?\n/)
  const out: string[] = []
  for (const raw of lines) {
    const line = raw.trim()
    if (!line) continue
    if (/^TRIGGER\b/i.test(line) || /^SKIP\b/i.test(line) || /^WHEN\b/i.test(line)) {
      out.push(line)
    }
  }
  return out.slice(0, 6)
}

function formatRelative(ts: number): string {
  const secs = Math.max(1, Math.round((Date.now() - ts) / 1000))
  if (secs < 60) return `${secs}s ago`
  const mins = Math.round(secs / 60)
  if (mins < 60) return `${mins}m ago`
  const hrs = Math.round(mins / 60)
  return `${hrs}h ago`
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

    // Body editor state.
    const draft = signal('')
    const baseline = signal('')
    const saving = signal(false)
    const saveBanner = signal<Banner>(null)

    // Per-section editor state.
    const descDraft = signal('')
    const descBaseline = signal('')
    const descSaving = signal(false)
    const descBanner = signal<Banner>(null)
    const descSavedAt = signal<number | null>(null)
    // Re-evaluate "saved Xs ago" once a second so the label stays current
    // without a heavyweight scheduler.
    const nowTick = signal(Date.now())
    effect(() => {
      if (typeof window === 'undefined') return
      const t = setInterval(() => nowTick.set(Date.now()), 1000)
      return () => clearInterval(t)
    })

    const tagsList = signal<string[]>([])
    const tagsBaseline = signal<string[]>([])
    const tagsInput = signal('')
    const tagsSaving = signal(false)
    const tagsBanner = signal<Banner>(null)

    // Per-skill open-state for the metadata panel sections.
    const openMap = signal<Record<SectionId, boolean>>({ ...DEFAULT_OPEN })
    effect(() => {
      const id = skillId()
      openMap.set(loadOpenState(id))
    })

    function toggleSection(sec: SectionId, next: boolean): void {
      const cur = openMap()
      if (cur[sec] === next) return
      const updated = { ...cur, [sec]: next }
      openMap.set(updated)
      saveOpenState(skillId(), updated)
    }
    const isOpen = (sec: SectionId) => derived(() => !!openMap()[sec])

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

    // Seed per-section drafts whenever the loaded skill changes.
    effect(() => {
      const sk = skill()
      if (sk.kind !== 'ok' || !sk.value) return
      const desc = sk.value.description ?? ''
      descDraft.set(desc)
      descBaseline.set(desc)
      descBanner.set(null)
      descSavedAt.set(null)

      const tags = [...(sk.value.tags ?? [])]
      tagsList.set(tags)
      tagsBaseline.set(tags)
      tagsInput.set('')
      tagsBanner.set(null)
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
    // Stable view of the loaded skill that survives mid-flight churn while
    // `live()` re-subscribes between sibling skill rows.
    const sk = derived<SkillDetail | null>(() => {
      const s = skill()
      return s.kind === 'ok' && s.value ? s.value : null
    })
    const skName = derived(() => sk()?.name ?? '')
    const skId = derived(() => sk()?.id ?? '')
    const skKind = derived(() => sk()?.kind ?? '')
    const skSource = derived(() => sk()?.source ?? '')
    const skSourceLabel = derived(() => sourceLabel(sk()?.source ?? ''))
    const skBody = derived(() => sk()?.body ?? '')
    const skVariables = derived<string[]>(() => sk()?.variables ?? [])
    const skFiles = derived<string[]>(() => sk()?.files ?? [])
    const skAuthors = derived<string[]>(() => sk()?.authors ?? [])
    const skAuthorsText = derived(() => skAuthors().join(', '))

    const skTriggerHints = derived<string[]>(() => extractTriggerHints(sk()?.body ?? ''))
    const skReferences = derived<string[]>(() =>
      (sk()?.files ?? []).filter((f) => f.startsWith('references/') || f.startsWith('reference/')),
    )

    const descDirty = derived(() => descDraft() !== descBaseline())
    const tagsDirty = derived(() => {
      const a = tagsList(), b = tagsBaseline()
      if (a.length !== b.length) return true
      for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return true
      return false
    })

    const descSavedAgo = derived<string>(() => {
      const at = descSavedAt()
      if (!at) return ''
      nowTick() // re-evaluate every tick
      return formatRelative(at)
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

    function onDescInput(evt: Event): void {
      const el = evt.target as HTMLTextAreaElement
      if (typeof el.value === 'string') descDraft.set(el.value)
    }

    function onTagsInput(evt: Event): void {
      const el = evt.target as HTMLInputElement
      tagsInput.set(el.value)
    }

    function addTag(): void {
      const raw = tagsInput().trim().replace(/,+$/, '').trim()
      if (!raw) return
      // Allow paste-style comma input — split into multiple chips.
      const parts = raw.split(',').map((t) => t.trim()).filter(Boolean)
      const current = tagsList()
      const next = [...current]
      for (const p of parts) if (!next.includes(p)) next.push(p)
      tagsList.set(next)
      tagsInput.set('')
    }

    function onTagsKeydown(evt: KeyboardEvent): void {
      if (evt.key === 'Enter' || evt.key === ',') {
        evt.preventDefault()
        addTag()
      } else if (evt.key === 'Backspace' && tagsInput() === '' && tagsList().length > 0) {
        // Convenience: backspace on empty input pops the last chip into the
        // input so the user can edit-then-re-add it.
        const cur = tagsList()
        const last = cur[cur.length - 1]
        tagsList.set(cur.slice(0, -1))
        tagsInput.set(last)
      }
    }

    function removeTag(t: string): void {
      tagsList.set(tagsList().filter((x) => x !== t))
    }

    function resetDesc(): void {
      descDraft.set(descBaseline())
      descBanner.set(null)
    }

    function resetTags(): void {
      tagsList.set([...tagsBaseline()])
      tagsInput.set('')
      tagsBanner.set(null)
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

    // Replace-style frontmatter PUT — rebuild from current detail so we
    // preserve name + any unknown keys the daemon parses.
    async function putFrontmatter(
      partial: { description?: string; tags?: string[] },
    ): Promise<{ ok: true } | { ok: false; msg: string }> {
      const id = skillId()
      const sk = skill()
      if (!id || sk.kind !== 'ok' || !sk.value) return { ok: false, msg: 'Skill not loaded.' }
      const fm: Record<string, unknown> = {
        name: sk.value.name,
        description: partial.description ?? sk.value.description ?? '',
      }
      const tags = partial.tags ?? sk.value.tags ?? []
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
        if (!resp.ok) return { ok: false, msg: text || `Save failed (${resp.status})` }
        return { ok: true }
      } catch (e) {
        return { ok: false, msg: (e as Error).message ?? 'Save failed.' }
      }
    }

    async function saveDesc(): Promise<void> {
      if (descSaving() || !descDirty()) return
      descSaving.set(true)
      descBanner.set(null)
      const draftVal = descDraft()
      const r = await putFrontmatter({ description: draftVal, tags: tagsBaseline() })
      if (r.ok) {
        descBaseline.set(draftVal)
        descSavedAt.set(Date.now())
        descBanner.set({ ok: true, msg: 'Saved.' })
      } else {
        descBanner.set({ ok: false, msg: r.msg })
      }
      descSaving.set(false)
    }

    async function saveTags(): Promise<void> {
      if (tagsSaving() || !tagsDirty()) return
      // Auto-commit any pending text in the input field before saving so the
      // user doesn't lose a half-typed chip.
      if (tagsInput().trim()) addTag()
      tagsSaving.set(true)
      tagsBanner.set(null)
      const next = tagsList()
      const r = await putFrontmatter({ description: descBaseline(), tags: next })
      if (r.ok) {
        tagsBaseline.set([...next])
        tagsBanner.set({ ok: true, msg: 'Saved.' })
      } else {
        tagsBanner.set({ ok: false, msg: r.msg })
      }
      tagsSaving.set(false)
    }

    function copyId(): void {
      const id = skId()
      if (!id || typeof navigator === 'undefined' || !navigator.clipboard) return
      navigator.clipboard.writeText(id).catch(() => { /* clipboard may be denied — silent */ })
    }

    return {
      skillId, q, kind, tag, selectedFile,
      rail, skill, fileContent, fileMetas,
      draft, dirty, saving, saveBanner, activeIsEditable, selectedFileName,
      sk, skName, skId, skKind, skSource, skSourceLabel, skBody, skVariables, skFiles,
      skAuthors, skAuthorsText, skTriggerHints, skReferences,
      // Per-section editor state.
      descDraft, descDirty, descSaving, descBanner, descSavedAgo,
      tagsList, tagsInput, tagsDirty, tagsSaving, tagsBanner,
      // Section open state. We expose both the bool (used in handlers /
      // tests) and an `Attr` variant that yields `''` when open and `null`
      // when closed — the shape `attr:open={…}` needs in the template,
      // since the SUI compiler rejects ternaries with `null` literals in
      // attribute expressions.
      openIdentity: isOpen('identity'),
      openDescription: isOpen('description'),
      openTags: isOpen('tags'),
      openTriggers: isOpen('triggers'),
      openVariables: isOpen('variables'),
      openReferences: isOpen('references'),
      openEvals: isOpen('evals'),
      // bindAttr treats `true`/`false`/`null` as the boolean-attribute
      // convention (true → present, anything else → absent). Empty string
      // also encodes to "absent", so we must pass `true` to set the
      // attribute, not `''`.
      openIdentityAttr: derived(() => (openMap().identity ? true : null)),
      openDescriptionAttr: derived(() => (openMap().description ? true : null)),
      openTagsAttr: derived(() => (openMap().tags ? true : null)),
      openTriggersAttr: derived(() => (openMap().triggers ? true : null)),
      openVariablesAttr: derived(() => (openMap().variables ? true : null)),
      openReferencesAttr: derived(() => (openMap().references ? true : null)),
      openEvalsAttr: derived(() => (openMap().evals ? true : null)),
      toggleSection,
      // Handlers.
      onEditorInput, save,
      onDescInput, resetDesc, saveDesc,
      onTagsInput, onTagsKeydown, addTag, removeTag, resetTags, saveTags,
      copyId,
    }
  },
})
