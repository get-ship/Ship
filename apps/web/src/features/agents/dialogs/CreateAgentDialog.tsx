import { useState } from 'react'
import { useNavigate } from '@tanstack/react-router'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter,
} from '@ship/primitives'
import { Input } from '@ship/primitives'
import { Button } from '@ship/primitives'
import { makeAgent } from '#/features/agents/make-agent'
import { buildTransferBundle } from '#/features/studio/build-transfer-bundle'
import { usePushBundle } from '#/features/studio/hooks/useWorkspaceConfig'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'
import { toast } from 'sonner'

interface CreateAgentDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CreateAgentDialog({ open, onOpenChange }: CreateAgentDialogProps) {
  const navigate = useNavigate()
  const pushBundle = usePushBundle()
  const { connected: isConnected } = useDaemon()

  const [name, setName] = useState('')
  const [description, setDescription] = useState('')

  const handleCreate = () => {
    if (!name.trim()) return
    const agent = makeAgent({
      profile: { id: '', name: name.trim(), description: description.trim() },
    })

    if (isConnected) {
      const bundle = buildTransferBundle(agent)
      pushBundle.mutate(bundle, {
        onSuccess: () => {
          onOpenChange(false)
          resetForm()
          void navigate({ to: '/studio/agents/$id', params: { id: agent.profile.id } })
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : 'Failed to create agent')
        },
      })
    } else {
      toast.error('Daemon not connected')
    }
  }

  const resetForm = () => {
    setName('')
    setDescription('')
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && name.trim() && isConnected) {
      e.preventDefault()
      handleCreate()
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>New agent</DialogTitle>
          <DialogDescription>
            Give it a name. Configure everything else on the detail page.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3 py-2">
          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Name</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="e.g. web-lane, rust-runtime, qa-engineer"
              autoFocus
            />
          </div>

          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Description</label>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Optional -- what does this agent specialize in?"
            />
          </div>

          {!isConnected && (
            <p className="text-[11px] text-amber-500">
              Daemon not connected. Start the daemon to create agents.
            </p>
          )}
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>Cancel</Button>
          <Button
            onClick={handleCreate}
            disabled={!name.trim() || !isConnected || pushBundle.isPending}
          >
            {pushBundle.isPending ? 'Creating...' : 'Create'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
