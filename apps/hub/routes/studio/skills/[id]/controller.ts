// Hub version of `apps/shell/routes/studio/skills/[id]/controller.ts` —
// the surface is byte-identical to the shell (the studio's skill-detail
// view is one of the surfaces this job ports verbatim) EXCEPT the
// import seams the hub cannot satisfy verbatim:
//
//   1. The `ops` import — the hub's typed unary client (`src/lib/client.ts`).
//      skills.* has no live mux resolver (see live_resolvers.rs), so BOTH the
//      detail read and the frontmatter/body writes go through `ops`
//      (createShipConnectClient); the read decodes the `{ json }` envelope.
//      (The hub's live SSE client `ship` is reserved for ops that have a
//      resolver — agents.summaries, collections.list, …)
//   2. The `routes` import from `routes.gen` — the path is the same
//      depth in both apps, so the relative path is unchanged.
import { controller, signal, derived, live, effect } from '@ship/sui-runtime'
import type { LiveSource, ViewScope } from '@ship/sui-runtime'
import { ops } from '../../../../src/lib/client'
import type { Scope } from './page.sui'

interface SkillDetail {
  id: string
  name: string
  description: string
  kind: string
  source: string
  tags: string[]
  authors: string[]
  variables: string[]
  files: string[]
  body: string
}
type LiveResult = { kind: string; value?: SkillDetail }
type Banner = { ok: boolean; msg: string } | null
type Section = 'identity' | 'description' | 'tags' | 'variables' | 'files'

const SOURCE_LABELS: Record<string, string> = {
  'PROJECT SKILLS': 'Project',
  'USER SKILLS': 'User',
  LIBRARY: 'Library',
}
const sourceLabel = (s: string): string => SOURCE_LABELS[s] ?? s

// Parse the comma/whitespace tag input into a deduped, appended list.
const appendTags = (input: string, current: string[]): string[] => {
  const raw = input.trim().replace(/,+$/, '').trim()
  if (!raw) return current
  const next = [...current]
  for (const p of raw.split(',').map((t) => t.trim()).filter(Boolean)) if (!next.includes(p)) next.push(p)
  return next
}

export default controller({
  scope: ({ route }: ViewScope<Scope>) => {
    const skillId = derived(() => route.params.id() ?? '')
    const skillRes = live(() =>
      ops.skills.detail({ id: skillId() || '__none__' }) as unknown as LiveSource<SkillDetail>,
    )
    const sk = derived<SkillDetail | null>(() => {
      const r = skillRes() as LiveResult
      if (r.kind !== 'ok' || !r.value) return null
      // Unary op → `{ json }` envelope; decode to the SkillDetail payload.
      const v = r.value as SkillDetail & { json?: string }
      if (typeof v.json === 'string') {
        try { return JSON.parse(v.json) as SkillDetail } catch { return null }
      }
      return v
    })
    const loading = derived(() => (skillRes() as LiveResult).kind !== 'ok')

    const skName = derived(() => sk()?.name ?? '')
    const skSourceLabel = derived(() => sourceLabel(sk()?.source ?? ''))
    const skVariables = derived<string[]>(() => sk()?.variables ?? [])
    const skFiles = derived<string[]>(() => sk()?.files ?? [])
    const skAuthorsText = derived(() => (sk()?.authors ?? []).join(', '))
    const hasBody = derived(() => (sk()?.body ?? '').length > 0)

    // Editor + per-section edit state.
    const draft = signal('')
    const baseline = signal('')
    const dirty = derived(() => draft() !== baseline())
    const saving = signal(false)
    const saveBanner = signal<Banner>(null)

    const descDraft = signal('')
    const descBaseline = signal('')
    const descDirty = derived(() => descDraft() !== descBaseline())
    const descSaving = signal(false)
    const descBanner = signal<Banner>(null)

    const tagsList = signal<string[]>([])
    const tagsBaseline = signal<string[]>([])
    const tagsInput = signal('')
    const tagsDirty = derived(() => {
      const a = tagsList(), b = tagsBaseline()
      return a.length !== b.length || a.some((t, i) => t !== b[i])
    })
    const tagsSaving = signal(false)
    const tagsBanner = signal<Banner>(null)

    // Seed editor + metadata drafts whenever a new skill loads.
    let seededId = ''
    effect(() => {
      const s = sk()
      if (s && s.id !== seededId) {
        seededId = s.id
        draft.set(s.body ?? ''); baseline.set(s.body ?? '')
        descDraft.set(s.description ?? ''); descBaseline.set(s.description ?? '')
        tagsList.set([...(s.tags ?? [])]); tagsBaseline.set([...(s.tags ?? [])])
      }
    })

    // Collapsible open state (in-memory; description + tags open by default).
    const open = signal<Record<Section, boolean>>({
      identity: true, description: true, tags: true, variables: false, files: false,
    })
    const openAttr = (s: Section) => derived(() => (open()[s] ? true : null))

    async function putFrontmatter(p: { description?: string; tags?: string[] }): Promise<Banner> {
      const id = skillId(); const s = sk()
      if (!id || !s) return { ok: false, msg: 'Skill not loaded.' }
      const fm: Record<string, unknown> = { name: s.name, description: p.description ?? s.description ?? '' }
      const tags = p.tags ?? s.tags ?? []
      if (tags.length) fm.tags = tags
      if ((s.authors ?? []).length) fm.authors = s.authors
      try {
        await ops.skills.writeFrontmatter({ id, frontmatterJson: JSON.stringify(fm), scope: '' })
        return { ok: true, msg: 'Saved.' }
      } catch (e) {
        return { ok: false, msg: (e as Error).message ?? 'Save failed.' }
      }
    }

    return {
      skillId, sk, loading, skName, skSourceLabel, skVariables, skFiles, skAuthorsText, hasBody,
      draft, baseline, dirty, saving, saveBanner,
      descDraft, descBaseline, descDirty, descSaving, descBanner,
      tagsList, tagsBaseline, tagsInput, tagsDirty, tagsSaving, tagsBanner,
      open, putFrontmatter,
      openIdentityAttr: openAttr('identity'),
      openDescriptionAttr: openAttr('description'),
      openTagsAttr: openAttr('tags'),
      openVariablesAttr: openAttr('variables'),
      openFilesAttr: openAttr('files'),
    }
  },

  on: {
    toggleSection: (s, name: string, isOpen: boolean) => {
      s.open = { ...s.open, [name]: isOpen }
    },
    onEditorInput: (s, e: Event) => {
      const d = (e as CustomEvent<{ value?: string }>).detail
      const v = d && typeof d.value === 'string' ? d.value : (e.target as HTMLElement & { value?: string }).value
      if (typeof v === 'string') s.draft = v
    },
    save: async (s) => {
      const id = s.skillId
      if (!id || s.saving) return
      s.saving = true; s.saveBanner = null
      try {
        await ops.skills.writeBody({ id, body: s.draft, scope: '' })
        s.baseline = s.draft
        s.saveBanner = { ok: true, msg: 'Saved.' }
      } catch (e) {
        s.saveBanner = { ok: false, msg: (e as Error).message ?? 'Save failed.' }
      } finally {
        s.saving = false
      }
    },
    onDescInput: (s, e: Event) => { s.descDraft = (e.target as HTMLTextAreaElement).value ?? '' },
    resetDesc: (s) => { s.descDraft = s.descBaseline; s.descBanner = null },
    saveDesc: async (s) => {
      if (s.descSaving || !s.descDirty) return
      s.descSaving = true; s.descBanner = null
      const r = await s.putFrontmatter({ description: s.descDraft, tags: s.tagsBaseline })
      if (r && r.ok) { s.descBaseline = s.descDraft; s.descBanner = { ok: true, msg: 'Saved.' } }
      else s.descBanner = r
      s.descSaving = false
    },
    onTagsInput: (s, e: Event) => { s.tagsInput = (e.target as HTMLInputElement).value },
    addTag: (s) => {
      const next = appendTags(s.tagsInput, s.tagsList)
      if (next !== s.tagsList) { s.tagsList = next; s.tagsInput = '' }
    },
    onTagsKeydown: (s, e: KeyboardEvent) => {
      if (e.key !== 'Enter' && e.key !== ',') return
      e.preventDefault()
      const next = appendTags(s.tagsInput, s.tagsList)
      if (next !== s.tagsList) { s.tagsList = next; s.tagsInput = '' }
    },
    removeTag: (s, t: string) => { s.tagsList = s.tagsList.filter((x: string) => x !== t) },
    resetTags: (s) => { s.tagsList = [...s.tagsBaseline]; s.tagsInput = ''; s.tagsBanner = null },
    saveTags: async (s) => {
      if (s.tagsSaving || !s.tagsDirty) return
      if (s.tagsInput.trim()) {
        const raw = s.tagsInput.trim().replace(/,+$/, '').trim()
        const next = [...s.tagsList]
        for (const p of raw.split(',').map((t: string) => t.trim()).filter(Boolean)) if (!next.includes(p)) next.push(p)
        s.tagsList = next; s.tagsInput = ''
      }
      s.tagsSaving = true; s.tagsBanner = null
      const next = s.tagsList
      const r = await s.putFrontmatter({ description: s.descBaseline, tags: next })
      if (r && r.ok) { s.tagsBaseline = [...next]; s.tagsBanner = { ok: true, msg: 'Saved.' } }
      else s.tagsBanner = r
      s.tagsSaving = false
    },
  },
})
