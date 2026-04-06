import { useState } from 'react'
import { X, MessageSquare, Activity } from 'lucide-react'
import { Button } from '@ship/primitives'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@ship/primitives'
import { useEventStream } from './events/useEventStream'
import { useDaemon } from './hooks/useDaemon'

interface AgentPanelProps {
  open: boolean
  onClose: () => void
}

export function AgentPanel({ open, onClose }: AgentPanelProps) {
  const [activeTab, setActiveTab] = useState<string>('chat')
  const { events } = useEventStream()
  const { workspaces } = useDaemon()
  const activeWs = workspaces.find((w) => String(w.status).toLowerCase() === 'active')

  if (!open) return null

  const studioEvents = events.filter((e) => e.event_type.startsWith('studio.'))

  return (
    <div className="fixed bottom-4 right-4 w-96 h-[28rem] z-50 bg-card border border-border/60 rounded-xl shadow-xl flex flex-col animate-in slide-in-from-bottom-2 duration-150">
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex flex-col flex-1 min-h-0 gap-0">
        <div className="flex items-center border-b border-border/40 px-2 shrink-0">
          <TabsList className="h-9 bg-transparent p-0 gap-0 flex-1">
            <TabsTrigger
              value="chat"
              className="flex items-center gap-1.5 h-full rounded-none border-b-2 text-[11px] font-medium data-[active]:border-primary data-[active]:text-primary border-transparent text-muted-foreground"
            >
              <MessageSquare className="size-3" />
              {activeWs?.active_agent ?? 'Chat'}
            </TabsTrigger>
            <TabsTrigger
              value="events"
              className="flex items-center gap-1.5 h-full rounded-none border-b-2 text-[11px] font-medium data-[active]:border-primary data-[active]:text-primary border-transparent text-muted-foreground"
            >
              <Activity className="size-3" />
              Events
            </TabsTrigger>
          </TabsList>
          <Button variant="ghost" size="icon" className="size-6 shrink-0" onClick={onClose}>
            <X className="size-3" />
          </Button>
        </div>

        <TabsContent value="chat" className="flex-1 min-h-0 m-0 p-0">
          <div className="flex flex-col h-full">
            <div className="flex-1 overflow-y-auto px-3 py-2 space-y-2">
              {studioEvents.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-full gap-2 text-center">
                  <MessageSquare className="size-5 text-muted-foreground/30" />
                  <p className="text-[11px] text-muted-foreground/50">No messages yet</p>
                </div>
              ) : (
                [...studioEvents].reverse().map((e, i) => (
                  <div key={i} className="text-xs text-muted-foreground">
                    <span className="text-primary/70">{e.event_type.replace('studio.', '')}</span>
                    {' '}{e.entity_id}
                  </div>
                ))
              )}
            </div>
          </div>
        </TabsContent>

        <TabsContent value="events" className="flex-1 min-h-0 m-0 p-0">
          <div className="flex-1 overflow-y-auto p-2 font-mono text-[10px] text-muted-foreground space-y-0.5 h-full">
            {events.length === 0 ? (
              <div className="text-center py-4 text-muted-foreground/60">No events yet</div>
            ) : (
              events.slice(0, 100).map((evt, i) => (
                <div key={i} className="flex gap-2 py-0.5">
                  <span className="text-muted-foreground/40 shrink-0">
                    {formatTime(evt.created_at)}
                  </span>
                  <span className="text-primary/80">{evt.event_type}</span>
                  <span className="truncate">{evt.entity_id}</span>
                </div>
              ))
            )}
          </div>
        </TabsContent>
      </Tabs>
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
