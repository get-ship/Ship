import { useState, useEffect } from 'react'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter,
} from '@ship/primitives'
import { Input } from '@ship/primitives'
import { Button } from '@ship/primitives'
import { useCreateJob } from '#/features/studio/hooks/useDaemon'
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

export function CreateJobDialog({ open, onOpenChange }: CreateJobDialogProps) {
  const createJob = useCreateJob()

  const [slug, setSlug] = useState('')
  const [agent, setAgent] = useState('')
  const [branch, setBranch] = useState('')
  const [specPath, setSpecPath] = useState('.ship-session/job-spec.md')
  const [dependsOn, setDependsOn] = useState('')
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

    const deps = dependsOn
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean)

    createJob.mutate(
      {
        slug: trimmedSlug,
        agent: agent.trim(),
        branch: branch.trim() || `job/${slugify(trimmedSlug)}`,
        spec_path: specPath.trim() || '.ship-session/job-spec.md',
        depends_on: deps.length > 0 ? deps : null,
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
    setSpecPath('.ship-session/job-spec.md')
    setDependsOn('')
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
              placeholder=".ship-session/job-spec.md"
            />
          </div>

          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Depends on</label>
            <Input
              value={dependsOn}
              onChange={(e) => setDependsOn(e.target.value)}
              placeholder="comma-separated slugs, e.g. setup-db, lint"
            />
            <p className="text-[11px] text-muted-foreground/60 mt-1">
              Jobs that must complete before this one is dispatched.
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
