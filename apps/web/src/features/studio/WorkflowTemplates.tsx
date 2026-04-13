import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { GitMerge, Plus, Save, ChevronRight, Trash2 } from 'lucide-react'
import { Button } from '@ship/primitives'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@ship/primitives'
import { Input } from '@ship/primitives'
import { usePlatform } from '#/features/studio/PlatformProvider'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'
import { daemonFetch } from '#/lib/daemon-config'

interface PipelineStep {
  id: string
  type: string
  agent?: string
  after: string[]
  model?: string
}

interface PipelineDef {
  name: string
  description: string
  steps: PipelineStep[]
  raw: string
}

function useWorkspaceId() {
  const { workspace } = usePlatform()
  const viewedWorkspace = workspace?.name ?? null
  const { workspaces } = useDaemon()
  return viewedWorkspace
    ?? (workspaces.find((w) => String(w.status).toLowerCase() === 'active') ?? workspaces[0])?.name
    ?? ''
}

function usePipelines() {
  const wsId = useWorkspaceId()
  return useQuery({
    queryKey: ['pipelines', wsId],
    queryFn: async (): Promise<PipelineDef[]> => {
      const res = await daemonFetch(`/api/workspaces/${encodeURIComponent(wsId)}/pipelines`)
      if (!res.ok) return []
      const body = await res.json() as { ok: boolean; data: { pipelines: PipelineDef[] } }
      return body.data.pipelines
    },
    enabled: wsId.length > 0,
    staleTime: 10_000,
  })
}

function usePipelineMutations() {
  const wsId = useWorkspaceId()
  const queryClient = useQueryClient()
  const invalidate = () => void queryClient.invalidateQueries({ queryKey: ['pipelines'] })
  const save = useMutation({
    mutationFn: async ({ name, content }: { name: string; content: string }) => {
      const res = await daemonFetch(
        `/api/workspaces/${encodeURIComponent(wsId)}/pipelines/${encodeURIComponent(name)}`,
        { method: 'PUT', body: JSON.stringify({ content }) },
      )
      if (!res.ok) throw new Error(`save pipeline: ${res.status}`)
    },
    onSuccess: invalidate,
  })
  const del = useMutation({
    mutationFn: async (name: string) => {
      const res = await daemonFetch(
        `/api/workspaces/${encodeURIComponent(wsId)}/pipelines/${encodeURIComponent(name)}`,
        { method: 'DELETE' },
      )
      if (!res.ok) throw new Error(`delete pipeline: ${res.status}`)
    },
    onSuccess: invalidate,
  })
  return { save, del }
}

const STEP_TYPE_COLORS: Record<string, string> = {
  agent: 'bg-sky-500/10 text-sky-600 border-sky-500/20',
  gate: 'bg-amber-500/10 text-amber-600 border-amber-500/20',
  merge: 'bg-emerald-500/10 text-emerald-600 border-emerald-500/20',
}

export function WorkflowTemplates() {
  const { data: pipelines, isLoading } = usePipelines()
  const [selected, setSelected] = useState<string | null>(null)
  const [editing, setEditing] = useState<string | null>(null)
  const [editorContent, setEditorContent] = useState('')
  const [createOpen, setCreateOpen] = useState(false)
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null)
  const { save: saveMutation, del: deleteMutation } = usePipelineMutations()

  const handleEdit = (pipeline: PipelineDef) => {
    setEditing(pipeline.name)
    setEditorContent(pipeline.raw)
    setSelected(pipeline.name)
  }

  const handleSave = () => {
    if (!editing) return
    saveMutation.mutate({ name: editing, content: editorContent }, {
      onSuccess: () => setEditing(null),
    })
  }

  return (
    <>
      <div className="flex items-center justify-between mb-4">
        <p className="text-xs text-muted-foreground">
          Pipeline templates define multi-step agent workflows.
        </p>
        <Button size="sm" variant="outline" onClick={() => setCreateOpen(true)}>
          <Plus className="size-3" />
          New Template
        </Button>
      </div>

      {isLoading ? (
        <div className="text-sm text-muted-foreground">Loading...</div>
      ) : !pipelines?.length ? (
        <div className="flex flex-col items-center py-16 text-center">
          <GitMerge className="size-8 text-muted-foreground/30 mb-3" />
          <p className="text-sm font-medium">No pipeline templates</p>
          <p className="text-xs text-muted-foreground mt-1">
            Create a pipeline in .ship/pipelines/
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {pipelines.map((pipeline) => (
            <PipelineCard
              key={pipeline.name}
              pipeline={pipeline}
              isSelected={selected === pipeline.name}
              isEditing={editing === pipeline.name}
              editorContent={editorContent}
              confirmingDelete={confirmDelete === pipeline.name}
              onToggle={() => setSelected(selected === pipeline.name ? null : pipeline.name)}
              onEdit={() => handleEdit(pipeline)}
              onEditorChange={setEditorContent}
              onSave={handleSave}
              onCancelEdit={() => setEditing(null)}
              onRequestDelete={() => setConfirmDelete(pipeline.name)}
              onConfirmDelete={() => { deleteMutation.mutate(pipeline.name); setConfirmDelete(null) }}
              onCancelDelete={() => setConfirmDelete(null)}
              saving={saveMutation.isPending}
            />
          ))}
        </div>
      )}

      <CreatePipelineDialog open={createOpen} onOpenChange={setCreateOpen} />
    </>
  )
}

interface PipelineCardProps {
  pipeline: PipelineDef; isSelected: boolean; isEditing: boolean
  editorContent: string; confirmingDelete: boolean; saving: boolean
  onToggle: () => void; onEdit: () => void; onEditorChange: (v: string) => void
  onSave: () => void; onCancelEdit: () => void
  onRequestDelete: () => void; onConfirmDelete: () => void; onCancelDelete: () => void
}

function PipelineCard(props: PipelineCardProps) {
  const { pipeline } = props

  return (
    <div className="rounded-lg border border-border/60 bg-card overflow-hidden">
      <div
        className="flex items-center gap-3 px-4 py-3 cursor-pointer hover:bg-muted/20 transition-colors"
        onClick={props.onToggle}
      >
        <GitMerge className="size-4 text-muted-foreground shrink-0" />
        <div className="flex-1 min-w-0">
          <span className="text-sm font-semibold">{pipeline.name}</span>
          {pipeline.description && (
            <p className="text-xs text-muted-foreground/60 mt-0.5 truncate">
              {pipeline.description}
            </p>
          )}
        </div>
        <span className="text-[10px] text-muted-foreground/40">
          {pipeline.steps.length} steps
        </span>
        <ChevronRight className={`size-3.5 text-muted-foreground/40 transition-transform ${props.isSelected ? 'rotate-90' : ''}`} />
      </div>

      {props.isSelected && (
        <div className="border-t border-border/40 px-4 py-3">
          <div className="flex items-center gap-2 flex-wrap mb-4">
            {pipeline.steps.map((step, i) => (
              <div key={step.id} className="flex items-center gap-2">
                <div className={`px-3 py-1.5 rounded-lg border text-xs font-medium ${STEP_TYPE_COLORS[step.type] ?? 'bg-muted/30 text-muted-foreground border-border/40'}`}>
                  <div>{step.id}</div>
                  <div className="text-[9px] opacity-60 mt-0.5">
                    {step.type}{step.agent ? ` \u00b7 ${step.agent}` : ''}{step.model ? ` \u00b7 ${step.model}` : ''}
                  </div>
                </div>
                {i < pipeline.steps.length - 1 && (
                  <ChevronRight className="size-3 text-muted-foreground/30" />
                )}
              </div>
            ))}
          </div>

          {props.isEditing ? (
            <div className="space-y-2">
              <textarea
                value={props.editorContent}
                onChange={(e) => props.onEditorChange(e.target.value)}
                className="w-full h-48 rounded-md border border-border bg-background px-3 py-2 text-xs font-mono resize-none focus:outline-none focus:border-primary/50"
                spellCheck={false}
              />
              <div className="flex gap-2">
                <Button size="sm" onClick={props.onSave} disabled={props.saving}>
                  <Save className="size-3" />
                  {props.saving ? 'Saving...' : 'Save'}
                </Button>
                <Button size="sm" variant="ghost" onClick={props.onCancelEdit}>
                  Cancel
                </Button>
              </div>
            </div>
          ) : (
            <div className="flex items-center gap-2">
              <Button size="sm" variant="ghost" onClick={props.onEdit}>
                Edit YAML
              </Button>
              {props.confirmingDelete ? (
                <span className="flex items-center gap-1.5">
                  <Button size="sm" variant="ghost" className="text-destructive hover:text-destructive" onClick={props.onConfirmDelete}>
                    Confirm delete
                  </Button>
                  <Button size="sm" variant="ghost" onClick={props.onCancelDelete}>
                    Cancel
                  </Button>
                </span>
              ) : (
                <Button size="sm" variant="ghost" onClick={props.onRequestDelete}>
                  <Trash2 className="size-3 text-muted-foreground hover:text-destructive" />
                </Button>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  )
}

function CreatePipelineDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (o: boolean) => void }) {
  const [name, setName] = useState('')
  const { save: saveMutation } = usePipelineMutations()

  const n = name || 'my-pipeline'
  const defaultYaml = [
    `name: ${n}`, 'description: \'\'', '',
    'steps:', '  - id: implement', '    type: agent', '    agent: default', '',
    '  - id: gate', '    type: gate', '    agent: gatekeeper', '    after: [implement]', '',
    '  - id: merge', '    type: merge', '    after: [gate]', '',
  ].join('\n')

  const handleCreate = () => {
    if (!name.trim()) return
    saveMutation.mutate({ name: name.trim(), content: defaultYaml }, {
      onSuccess: () => {
        onOpenChange(false)
        setName('')
      },
    })
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>New pipeline template</DialogTitle>
        </DialogHeader>
        <div className="py-2">
          <label className="text-xs font-medium text-foreground mb-1.5 block">Name</label>
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g. tdd, review-only, full-ci"
            autoFocus
          />
          <p className="text-[11px] text-muted-foreground/60 mt-1">
            Creates .ship/pipelines/{name || '...'}.yaml
          </p>
        </div>
        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>Cancel</Button>
          <Button onClick={handleCreate} disabled={!name.trim() || saveMutation.isPending}>
            {saveMutation.isPending ? 'Creating...' : 'Create'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
