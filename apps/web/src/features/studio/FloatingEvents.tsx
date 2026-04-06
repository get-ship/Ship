import { X, Activity } from 'lucide-react'
import { Button } from '@ship/primitives'
import { useQueryClient } from '@tanstack/react-query'
import { mcpKeys } from '#/lib/query-keys'
import type { EventEnvelope } from '#/features/studio/events/useEventStream'
import { useStudioShell } from './StudioShellContext'

export function FloatingEvents() {
  const { eventsOpen, toggleEvents } = useStudioShell()
  const queryClient = useQueryClient()
  const events = queryClient.getQueryData<EventEnvelope[]>(mcpKeys.events()) ?? []

  if (!eventsOpen) return null

  return (
    <div className="fixed bottom-4 right-4 w-96 max-h-80 z-50 bg-card border border-border/60 rounded-xl shadow-xl flex flex-col animate-in slide-in-from-bottom-2 duration-150">
      <div className="flex items-center justify-between px-3 py-2 border-b border-border/40">
        <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
          <Activity className="size-3" />
          Events
        </div>
        <Button variant="ghost" size="icon" className="size-6" onClick={toggleEvents}>
          <X className="size-3" />
        </Button>
      </div>
      <div className="flex-1 overflow-auto p-2 font-mono text-[10px] text-muted-foreground space-y-0.5">
        {events.length === 0 && (
          <div className="text-center py-4 text-muted-foreground/60">No events yet</div>
        )}
        {events.slice(0, 50).map((evt, i) => (
          <div key={i} className="flex gap-2 py-0.5">
            <span className="text-muted-foreground/40 shrink-0">{formatTime(evt.created_at)}</span>
            <span className="text-primary/80">{evt.event_type}</span>
            <span className="truncate">{evt.entity_id}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

function formatTime(iso: string): string {
  try {
    return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  } catch {
    return ''
  }
}
