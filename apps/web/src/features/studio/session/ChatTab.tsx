// Agent chat — scoped to the viewed workspace's agent.
// Shows: user messages (studio.message), agent artifacts (session file writes),
// and session progress events as a conversation thread.

import { useRef, useEffect, useMemo } from 'react'
import { MessageSquare, FileText, Bot, User, AlertTriangle } from 'lucide-react'
import { useEventStream } from '../events/useEventStream'
import { ChatDraftArea } from './ChatDraftArea'
import { usePlatform } from '#/features/studio/PlatformProvider'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'
import type { StagedAnnotation } from './types'
import type { EventEnvelope } from '../events/useEventStream'

interface Props {
  stagedAnnotations: StagedAnnotation[]
  onSend: (text: string) => Promise<void>
  onRemoveAnnotation: (id: string) => void
  onUploadFiles: (files: FileList) => void
  disabled?: boolean
}

interface ChatMessage {
  id: string
  sender: 'user' | 'agent' | 'system'
  content: string
  detail?: string
  timestamp: string
  eventType: string
  filePath?: string
}

function eventToMessage(e: EventEnvelope, viewedWs: string | null): ChatMessage | null {
  // Filter to events for the viewed workspace
  if (viewedWs && e.workspace_id && e.workspace_id !== viewedWs) return null

  let payload: Record<string, unknown> | null = null
  if (e.payload_json) {
    try { payload = JSON.parse(e.payload_json) as Record<string, unknown> } catch { /* skip */ }
  }

  const ts = new Date(e.created_at).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })

  // User messages
  if (e.event_type === 'studio.message') {
    const summary = payload?.summary as string | undefined
    const annCount = (payload?.annotations as unknown[])?.length ?? 0
    const content = summary || (annCount > 0 ? `Sent ${annCount} annotation${annCount !== 1 ? 's' : ''}` : 'Message sent')
    return { id: e.id, sender: 'user', content, timestamp: ts, eventType: e.event_type }
  }

  // Agent progress
  if (e.event_type === 'session.progress') {
    const msg = payload?.message as string | undefined
    const escalation = payload?.escalation as string | undefined
    if (!msg) return null
    return {
      id: e.id,
      sender: 'agent',
      content: msg,
      detail: escalation ? `[${escalation}]` : undefined,
      timestamp: ts,
      eventType: e.event_type,
    }
  }

  // Session started/ended
  if (e.event_type === 'session.started') {
    const agent = payload?.agent_id as string | undefined
    return { id: e.id, sender: 'system', content: `Session started${agent ? ` (${agent})` : ''}`, timestamp: ts, eventType: e.event_type }
  }
  if (e.event_type === 'session.ended') {
    const summary = payload?.summary as string | undefined
    return { id: e.id, sender: 'system', content: summary ?? 'Session ended', timestamp: ts, eventType: e.event_type }
  }

  // Job status changes
  if (e.event_type.startsWith('job.')) {
    const slug = payload?.slug as string | undefined
    const action = e.event_type.replace('job.', '')
    return { id: e.id, sender: 'system', content: `Job ${slug ?? e.entity_id}: ${action}`, timestamp: ts, eventType: e.event_type }
  }

  return null
}

export function ChatTab({
  stagedAnnotations,
  onSend,
  onRemoveAnnotation,
  onUploadFiles,
  disabled = false,
}: Props) {
  const { events } = useEventStream()
  const { workspace } = usePlatform()
  const viewedWorkspace = workspace?.name ?? null
  const { workspaces } = useDaemon()
  const bottomRef = useRef<HTMLDivElement>(null)

  const currentWs = viewedWorkspace
    ? workspaces.find((w) => w.name === viewedWorkspace)
    : workspaces.find((w) => String(w.status).toLowerCase() === 'active') ?? workspaces[0]

  const messages = useMemo(() => {
    return events
      .map((e) => eventToMessage(e, currentWs?.id ?? null))
      .filter(Boolean) as ChatMessage[]
  }, [events, currentWs?.id])

  // Reverse so oldest first (events are most-recent-first)
  const thread = useMemo(() => [...messages].reverse(), [messages])

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [thread.length])

  return (
    <div className="flex flex-col flex-1 min-h-0">
      {/* Agent identity header */}
      <div className="flex items-center gap-2 px-3 py-1.5 border-b border-border/30 shrink-0">
        <Bot className="size-3 text-primary/60" />
        <span className="text-[10px] font-medium text-muted-foreground truncate">
          {currentWs?.active_agent ?? 'No agent'}
        </span>
        <span className="text-[10px] text-muted-foreground/60 truncate">
          {currentWs?.name ?? 'No workspace'}
        </span>
      </div>

      {/* Thread */}
      <div className="flex-1 overflow-y-auto min-h-0 px-3 py-2 space-y-2">
        {thread.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full gap-2 py-8 text-center">
            <MessageSquare className="size-6 text-muted-foreground/60" />
            <p className="text-[11px] text-muted-foreground/50">No messages yet</p>
            <p className="text-[10px] text-muted-foreground/60">
              Annotate artifacts and send to agent
            </p>
          </div>
        ) : (
          thread.map((msg) => <ChatBubble key={msg.id} message={msg} />)
        )}
        <div ref={bottomRef} />
      </div>

      <ChatDraftArea
        stagedAnnotations={stagedAnnotations}
        onSend={onSend}
        onRemoveAnnotation={onRemoveAnnotation}
        onUploadFiles={onUploadFiles}
        disabled={disabled}
      />
    </div>
  )
}

function ChatBubble({ message }: { message: ChatMessage }) {
  if (message.sender === 'system') {
    return (
      <div className="flex items-center gap-2 py-1">
        <div className="flex-1 border-t border-border/20" />
        <span className="text-[9px] text-muted-foreground/60 shrink-0">{message.content}</span>
        <div className="flex-1 border-t border-border/20" />
      </div>
    )
  }

  const isUser = message.sender === 'user'

  return (
    <div className={`flex gap-2 ${isUser ? 'flex-row-reverse' : ''}`}>
      <div className={`size-5 rounded-full flex items-center justify-center shrink-0 mt-0.5 ${
        isUser ? 'bg-primary/10' : 'bg-muted/50'
      }`}>
        {isUser
          ? <User className="size-2.5 text-primary" />
          : <Bot className="size-2.5 text-muted-foreground" />}
      </div>
      <div className={`max-w-[80%] rounded-lg px-2.5 py-1.5 ${
        isUser
          ? 'bg-primary/10 text-foreground'
          : 'bg-muted/30 text-foreground'
      }`}>
        <p className="text-[11px] leading-relaxed">{message.content}</p>
        {message.detail && (
          <div className="flex items-center gap-1 mt-1">
            <AlertTriangle className="size-2.5 text-amber-500" />
            <span className="text-[9px] text-amber-500 font-medium">{message.detail}</span>
          </div>
        )}
        {message.filePath && (
          <div className="flex items-center gap-1 mt-1">
            <FileText className="size-2.5 text-muted-foreground/60" />
            <span className="text-[9px] text-muted-foreground/60">{message.filePath}</span>
          </div>
        )}
        <span className="text-[8px] text-muted-foreground/60 mt-0.5 block">{message.timestamp}</span>
      </div>
    </div>
  )
}
