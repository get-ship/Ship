import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { GitMerge, Plus, Save, ChevronRight, Trash2 } from 'lucide-react'
import { Button } from '@ship/primitives'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@ship/primitives'
import { Input } from '@ship/primitives'
import { daemonFetch } from '#/lib/daemon-config'
import { useStudioShell } from '#/features/studio/StudioShellContext'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'

export const Route = createFileRoute('/studio/pipelines')({
  component: PipelinesPage,
  ssr: false,
})

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

function usePipelines() {
  const { viewedWorkspace } = useStudioShell()
  const { workspaces } = useDaemon()
  const wsId = viewedWorkspace
    ?? workspaces.find((w) => String(w.status).toLowerCase() === 'active')?.branch
    ?? ''

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

function useSavePipeline() {
  const { viewedWorkspace } = useStudioShell()
  const { workspaces } = useDaemon()
  const wsId = viewedWorkspace
    ?? workspaces.find((w) => String(w.status).toLowerCase() === 'active')?.branch
    ?? ''
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ name, content }: { name: string; content: string }) => {
      const res = await daemonFetch(
        `/api/workspaces/${encodeURIComponent(wsId)}/pipelines/${encodeURIComponent(name)}`,
        { method: 'PUT', body: JSON.stringify({ content }) },
      )
      if (!res.ok) throw new Error(`save pipeline: ${res.status}`)
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['pipelines'] })
    },
  })
}

const STEP_TYPE_COLORS: Record<string, string> = {
  agent: 'bg-sky-500/10 text-sky-600 border-sky-500/20',
  gate: 'bg-amber-500/10 text-amber-600 border-amber-500/20',
  merge: 'bg-emerald-500/10 text-emerald-600 border-emerald-500/20',
}

function useDeletePipeline() {
  const { viewedWorkspace } = useStudioShell()
  const { workspaces } = useDaemon()
  const wsId = viewedWorkspace
    ?? workspaces.find((w) => String(w.status).toLowerCase() === 'active')?.branch
    ?? ''
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (name: string) => {
      const res = await daemonFetch(
        `/api/workspaces/${encodeURIComponent(wsId)}/pipelines/${encodeURIComponent(name)}`,
        { method: 'DELETE' },
      )
      if (!res.ok) throw new Error(`delete pipeline: ${res.status}`)
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['pipelines'] })
    },
  })
}

function PipelinesPage() {
  const { data: pipelines, isLoading } = usePipelines()
  const [selected, setSelected] = useState<string | null>(null)
  const [editing, setEditing] = useState<string | null>(null)
  const [editorContent, setEditorContent] = useState('')
  const [createOpen, setCreateOpen] = useState(false)
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null)
  const saveMutation = useSavePipeline()
  const deleteMutation = useDeletePipeline()

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
    <div className="flex-1 overflow-auto">
      <div className="max-w-5xl mx-auto px-6 py-8">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="font-display text-2xl font-bold text-foreground">Pipelines</h1>
            <p className="text-sm text-muted-foreground mt-1">
              Workflow templates for job execution
            </p>
          </div>
          <Button onClick={() => setCreateOpen(true)}>
            <Plus className="size-3.5" />
            New Pipeline
          </Button>
        </div>

        {isLoading ? (
          <div className="text-sm text-muted-foreground">Loading...</div>
        ) : !pipelines?.length ? (
          <div className="flex flex-col items-center py-16 text-center">
            <GitMerge className="size-8 text-muted-foreground/30 mb-3" />
            <p className="text-sm font-medium">No pipelines</p>
            <p className="text-xs text-muted-foreground mt-1">Create a pipeline in .ship/pipelines/</p>
          </div>
        ) : (
          <div className="space-y-4">
            {pipelines.map((pipeline) => (
              <div key={pipeline.name} className="rounded-lg border border-border/60 bg-card overflow-hidden">
                {/* Header */}
                <div
                  className="flex items-center gap-3 px-4 py-3 cursor-pointer hover:bg-muted/20 transition-colors"
                  onClick={() => setSelected(selected === pipeline.name ? null : pipeline.name)}
                >
                  <GitMerge className="size-4 text-muted-foreground shrink-0" />
                  <div className="flex-1 min-w-0">
                    <span className="text-sm font-semibold">{pipeline.name}</span>
                    {pipeline.description && (
                      <p className="text-xs text-muted-foreground/60 mt-0.5 truncate">{pipeline.description}</p>
                    )}
                  </div>
                  <span className="text-[10px] text-muted-foreground/40">{pipeline.steps.length} steps</span>
                  <ChevronRight className={`size-3.5 text-muted-foreground/40 transition-transform ${selected === pipeline.name ? 'rotate-90' : ''}`} />
                </div>

                {/* Expanded: step visualization */}
                {selected === pipeline.name && (
                  <div className="border-t border-border/40 px-4 py-3">
                    {/* Step flow */}
                    <div className="flex items-center gap-2 flex-wrap mb-4">
                      {pipeline.steps.map((step, i) => (
                        <div key={step.id} className="flex items-center gap-2">
                          <div className={`px-3 py-1.5 rounded-lg border text-xs font-medium ${STEP_TYPE_COLORS[step.type] ?? 'bg-muted/30 text-muted-foreground border-border/40'}`}>
                            <div>{step.id}</div>
                            <div className="text-[9px] opacity-60 mt-0.5">
                              {step.type}{step.agent ? ` · ${step.agent}` : ''}{step.model ? ` · ${step.model}` : ''}
                            </div>
                          </div>
                          {i < pipeline.steps.length - 1 && (
                            <ChevronRight className="size-3 text-muted-foreground/30" />
                          )}
                        </div>
                      ))}
                    </div>

                    {/* Editor */}
                    {editing === pipeline.name ? (
                      <div className="space-y-2">
                        <textarea
                          value={editorContent}
                          onChange={(e) => setEditorContent(e.target.value)}
                          className="w-full h-48 rounded-md border border-border bg-background px-3 py-2 text-xs font-mono resize-none focus:outline-none focus:border-primary/50"
                          spellCheck={false}
                        />
                        <div className="flex gap-2">
                          <Button size="sm" onClick={handleSave} disabled={saveMutation.isPending}>
                            <Save className="size-3" />
                            {saveMutation.isPending ? 'Saving...' : 'Save'}
                          </Button>
                          <Button size="sm" variant="ghost" onClick={() => setEditing(null)}>Cancel</Button>
                        </div>
                      </div>
                    ) : (
                      <div className="flex items-center gap-2">
                        <Button size="sm" variant="ghost" onClick={() => handleEdit(pipeline)}>
                          Edit YAML
                        </Button>
                        {confirmDelete === pipeline.name ? (
                          <span className="flex items-center gap-1.5">
                            <Button size="sm" variant="ghost" className="text-destructive hover:text-destructive" onClick={() => { deleteMutation.mutate(pipeline.name); setConfirmDelete(null) }}>
                              Confirm delete
                            </Button>
                            <Button size="sm" variant="ghost" onClick={() => setConfirmDelete(null)}>
                              Cancel
                            </Button>
                          </span>
                        ) : (
                          <Button size="sm" variant="ghost" onClick={() => setConfirmDelete(pipeline.name)}>
                            <Trash2 className="size-3 text-muted-foreground hover:text-destructive" />
                          </Button>
                        )}
                      </div>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}

        <CreatePipelineDialog open={createOpen} onOpenChange={setCreateOpen} />
      </div>
    </div>
  )
}

function CreatePipelineDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (o: boolean) => void }) {
  const [name, setName] = useState('')
  const saveMutation = useSavePipeline()

  const defaultYaml = `name: ${name || 'my-pipeline'}
description: ''

steps:
  - id: implement
    type: agent
    agent: default

  - id: gate
    type: gate
    agent: gatekeeper
    after: [implement]

  - id: merge
    type: merge
    after: [gate]
`

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
          <DialogTitle>New pipeline</DialogTitle>
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
