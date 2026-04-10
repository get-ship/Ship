import { useState, useEffect, useRef } from 'react'
import { useQuery } from '@tanstack/react-query'
import { X } from 'lucide-react'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter,
} from '@ship/primitives'
import { Input } from '@ship/primitives'
import { Button } from '@ship/primitives'
import { useCreateJob, useDaemon } from '#/features/studio/hooks/useDaemonData'
import { useStudioShell } from '#/features/studio/StudioShellContext'
import { daemonFetch } from '#/lib/daemon-config'
import { toast } from 'sonner'

interface CreateJobDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9-]/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '')
}

function usePipelineNames() {
  const { viewedWorkspace } = useStudioShell()
  const { workspaces } = useDaemon()
  const wsId = viewedWorkspace
    ?? workspaces.find((w) => String(w.status).toLowerCase() === 'active')?.branch
    ?? ''

  return useQuery({
    queryKey: ['pipeline-names', wsId],
    queryFn: async (): Promise<string[]> => {
      const res = await daemonFetch(`/api/workspaces/${encodeURIComponent(wsId)}/pipelines`)
      if (!res.ok) return []
      const body = await res.json() as { ok: boolean; data: { pipelines: Array<{ name: string }> } }
      return body.data.pipelines.map((p) => p.name)
    },
    enabled: wsId.length > 0,
    staleTime: 30_000,
  })
}

// ── DependsOnInput ─────────────────────────────────────────────────────────

function DependsOnInput({ value, onChange, suggestions }: {
  value: string[]
  onChange: (deps: string[]) => void
  suggestions: string[]
}) {
  const [input, setInput] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)

  const addSlug = (slug: string) => {
    const s = slug.trim()
    if (!s || value.includes(s)) { setInput(''); return }
    onChange([...value, s])
    setInput('')
    inputRef.current?.focus()
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault()
      addSlug(input)
    } else if (e.key === 'Backspace' && !input && value.length > 0) {
      onChange(value.slice(0, -1))
    }
  }

  // Datalist id must be unique to avoid collision
  const listId = 'job-slugs-datalist'

  return (
    <div
      className="min-h-9 flex flex-wrap gap-1 rounded-md border border-input bg-background px-3 py-1.5 cursor-text focus-within:border-primary/50 transition-colors"
      onClick={() => inputRef.current?.focus()}
    >
      {value.map((slug) => (
        <span key={slug} className="flex items-center gap-1 rounded bg-muted px-1.5 py-0.5 text-[11px] font-medium text-foreground/80">
          {slug}
          <button
            type="button"
            onClick={(e) => { e.stopPropagation(); onChange(value.filter((s) => s !== slug)) }}
            className="text-muted-foreground hover:text-destructive transition"
          >
            <X className="size-2.5" />
          </button>
        </span>
      ))}
      <input
        ref={inputRef}
        list={listId}
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={handleKeyDown}
        onBlur={() => { if (input.trim()) addSlug(input) }}
        placeholder={value.length === 0 ? 'Type a job slug…' : ''}
        className="flex-1 min-w-[120px] bg-transparent text-xs outline-none placeholder:text-muted-foreground/50"
      />
      <datalist id={listId}>
        {suggestions.map((s) => <option key={s} value={s} />)}
      </datalist>
    </div>
  )
}

// ── CreateJobDialog ────────────────────────────────────────────────────────

export function CreateJobDialog({ open, onOpenChange }: CreateJobDialogProps) {
  const createJob = useCreateJob()
  const { data: pipelineNames } = usePipelineNames()
  const { jobs } = useDaemon()
  const jobSlugs = jobs.map((j) => j.slug)

  const [slug, setSlug] = useState('')
  const [agent, setAgent] = useState('')
  const [branch, setBranch] = useState('')
  const [specPath, setSpecPath] = useState('.artifacts/job-spec.md')
  const [dependsOn, setDependsOn] = useState<string[]>([])
  const [pipeline, setPipeline] = useState('')
  const [branchEdited, setBranchEdited] = useState(false)

  useEffect(() => {
    if (!branchEdited && slug) {
      setBranch(`job/${slugify(slug)}`)
    }
  }, [slug, branchEdited])

  const handleBranchChange = (value: string) => {
    setBranch(value)
    setBranchEdited(true)
  }

  const handleSubmit = () => {
    const trimmedSlug = slug.trim()
    if (!trimmedSlug) return

    createJob.mutate(
      {
        slug: trimmedSlug,
        agent: agent.trim(),
        branch: branch.trim() || `job/${slugify(trimmedSlug)}`,
        spec_path: specPath.trim() || '.artifacts/job-spec.md',
        depends_on: dependsOn.length > 0 ? dependsOn : null,
        pipeline: pipeline || undefined,
      },
      {
        onSuccess: () => {
          toast.success(`Job "${trimmedSlug}" created`)
          onOpenChange(false)
          resetForm()
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : 'Failed to create job')
        },
      },
    )
  }

  const resetForm = () => {
    setSlug('')
    setAgent('')
    setBranch('')
    setSpecPath('.artifacts/job-spec.md')
    setDependsOn([])
    setPipeline('')
    setBranchEdited(false)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>New job</DialogTitle>
          <DialogDescription>
            Create a job to dispatch work to an agent.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">
              Slug <span className="text-destructive">*</span>
            </label>
            <Input
              value={slug}
              onChange={(e) => setSlug(e.target.value)}
              placeholder="e.g. auth-tests, fix-sidebar"
              autoFocus
            />
          </div>

          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Agent</label>
            <Input
              value={agent}
              onChange={(e) => setAgent(e.target.value)}
              placeholder="e.g. rust-runtime, web-lane"
            />
          </div>

          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Pipeline</label>
            <select
              value={pipeline}
              onChange={(e) => setPipeline(e.target.value)}
              className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground focus:outline-none focus:border-primary/50"
            >
              <option value="">Default (from spec frontmatter)</option>
              {pipelineNames?.map((name) => (
                <option key={name} value={name}>{name}</option>
              ))}
            </select>
          </div>

          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Branch</label>
            <Input
              value={branch}
              onChange={(e) => handleBranchChange(e.target.value)}
              placeholder="job/{slug}"
            />
            <p className="text-[11px] text-muted-foreground/60 mt-1">
              Auto-derived from slug. Edit to override.
            </p>
          </div>

          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Spec path</label>
            <Input
              value={specPath}
              onChange={(e) => setSpecPath(e.target.value)}
              placeholder=".artifacts/job-spec.md"
            />
          </div>

          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Depends on</label>
            <DependsOnInput
              value={dependsOn}
              onChange={setDependsOn}
              suggestions={jobSlugs}
            />
            <p className="text-[11px] text-muted-foreground/60 mt-1">
              Enter to add. Type to filter existing jobs.
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>Cancel</Button>
          <Button
            onClick={handleSubmit}
            disabled={!slug.trim() || createJob.isPending}
          >
            {createJob.isPending ? 'Creating...' : 'Create job'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
