import { X, MessageSquare } from 'lucide-react'
import { Button } from '@ship/primitives'
import { useStudioShell } from './StudioShellContext'
import { useDaemon } from './hooks/useDaemon'

export function FloatingChat() {
  const { chatOpen, toggleChat } = useStudioShell()
  const { workspaces } = useDaemon()
  const activeWs = workspaces.find((w) => String(w.status).toLowerCase() === 'active')

  if (!chatOpen) return null

  return (
    <div className="fixed bottom-4 right-4 w-96 h-96 z-50 bg-card border border-border/60 rounded-xl shadow-xl flex flex-col animate-in slide-in-from-bottom-2 duration-150">
      <div className="flex items-center justify-between px-3 py-2 border-b border-border/40">
        <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
          <MessageSquare className="size-3" />
          {activeWs?.active_agent ?? 'Agent Chat'}
        </div>
        <Button variant="ghost" size="icon" className="size-6" onClick={toggleChat}>
          <X className="size-3" />
        </Button>
      </div>
      <div className="flex-1 flex items-center justify-center text-xs text-muted-foreground/60">
        Scoped to workspace agent
      </div>
    </div>
  )
}
