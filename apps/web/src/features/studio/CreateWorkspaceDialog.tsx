import { useState, useEffect } from 'react'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter,
} from '@ship/primitives'
import { Input } from '@ship/primitives'
import { Button } from '@ship/primitives'
import { useDaemon, useCreateWorkspace } from '#/features/studio/hooks/useDaemon'
import { toast } from 'sonner'

interface CreateWorkspaceDialogProps {
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

export function CreateWorkspaceDialog({ open, onOpenChange }: CreateWorkspaceDialogProps) {
  const createWorkspace = useCreateWorkspace()
  const { agents } = useDaemon()

  const [name, setName] = useState('')
  const [branch, setBranch] = useState('')
  const [agent, setAgent] = useState('')
  const [baseBranch, setBaseBranch] = useState('main')
  const [branchEdited, setBranchEdited] = useState(false)

  useEffect(() => {
    if (!branchEdited && name) {
      setBranch(`feature/${slugify(name)}`)
    }
  }, [name, branchEdited])

  const handleBranchChange = (value: string) => {
    setBranch(value)
    setBranchEdited(true)
  }

  const resetForm = () => {
    setName('')
    setBranch('')
    setAgent('')
    setBaseBranch('main')
    setBranchEdited(false)
  }

  const handleSubmit = () => {
    const trimmedBranch = branch.trim()
    if (!trimmedBranch || !agent.trim()) return

    createWorkspace.mutate(
      {
        branch: trimmedBranch,
        agent_id: agent.trim(),
        base_branch: baseBranch.trim() || 'main',
      },
      {
        onSuccess: () => {
          toast.success(`Workspace "${trimmedBranch}" created`)
          onOpenChange(false)
          resetForm()
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : 'Failed to create workspace')
        },
      },
    )
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>New workspace</DialogTitle>
          <DialogDescription>
            Create a workspace with a git worktree.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">
              Name <span className="text-destructive">*</span>
            </label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. auth-flow, sidebar-redesign"
              autoFocus
            />
          </div>

          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">
              Agent <span className="text-destructive">*</span>
            </label>
            {agents.length > 0 ? (
              <select
                value={agent}
                onChange={(e) => setAgent(e.target.value)}
                className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
              >
                <option value="">Select an agent...</option>
                {agents.map((a) => (
                  <option key={a.agent_id} value={a.agent_id}>
                    {a.agent_id}
                  </option>
                ))}
              </select>
            ) : (
              <Input
                value={agent}
                onChange={(e) => setAgent(e.target.value)}
                placeholder="e.g. web-lane, rust-runtime"
              />
            )}
          </div>

          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Branch</label>
            <Input
              value={branch}
              onChange={(e) => handleBranchChange(e.target.value)}
              placeholder="feature/{name}"
            />
            <p className="text-[11px] text-muted-foreground/60 mt-1">
              Auto-derived from name. Edit to override.
            </p>
          </div>

          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Base branch</label>
            <Input
              value={baseBranch}
              onChange={(e) => setBaseBranch(e.target.value)}
              placeholder="main"
            />
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>Cancel</Button>
          <Button
            onClick={handleSubmit}
            disabled={!branch.trim() || !agent.trim() || createWorkspace.isPending}
          >
            {createWorkspace.isPending ? 'Creating...' : 'Create workspace'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
