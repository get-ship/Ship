// Right-side drawer: Chat + Events + Terminal tabs.
// State managed by StudioShellContext (Cmd+/ toggles, rail button toggles).

import { useState, useCallback } from 'react'
import { MessageSquare, Activity, TerminalIcon } from 'lucide-react'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@ship/primitives'
import { ChatTab } from './ChatTab'
import { EventStreamPanel } from '../events/EventStreamPanel'
import { TerminalTab } from './TerminalTab'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
import { useStudioShell } from '#/features/studio/StudioShellContext'
import type { StagedAnnotation } from './types'

interface Props {
  stagedAnnotations?: StagedAnnotation[]
  onSend?: (text: string) => Promise<void>
  onRemoveAnnotation?: (id: string) => void
  onUploadFiles?: (files: FileList) => void
  disabled?: boolean
}

export function RightDrawer({
  stagedAnnotations = [],
  onSend,
  onRemoveAnnotation,
  onUploadFiles,
  disabled = false,
}: Props) {
  const { drawerOpen } = useStudioShell()
  const [activeTab, setActiveTab] = useState<string>('chat')
  const { workspaces } = useDaemon()

  const activeWorkspaceId = workspaces.find((w) => String(w.status).toLowerCase() === 'active')?.id ?? null

  const noopSend = useCallback(async (_text: string) => {}, [])
  const noopRemove = useCallback((_id: string) => {}, [])
  const noopUpload = useCallback((_files: FileList) => {}, [])

  if (!drawerOpen) return null

  return (
    <div className="w-80 shrink-0 flex flex-col border-l border-border/60 bg-card/30 min-h-0 overflow-hidden">
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex flex-col flex-1 min-h-0 gap-0">
        <TabsList className="h-9 w-full rounded-none border-b border-border/60 bg-transparent p-0 justify-start gap-0 shrink-0">
          <TabsTrigger
            value="chat"
            className="flex items-center gap-1.5 flex-1 h-full rounded-none border-b-2 text-[11px] font-medium data-[active]:border-primary data-[active]:text-primary border-transparent text-muted-foreground"
          >
            <MessageSquare className="size-3" />
            Chat
            {stagedAnnotations.length > 0 && (
              <span className="ml-0.5 flex size-4 items-center justify-center rounded-full bg-primary text-[9px] font-bold text-primary-foreground">
                {stagedAnnotations.length > 9 ? '9+' : stagedAnnotations.length}
              </span>
            )}
          </TabsTrigger>
          <TabsTrigger
            value="events"
            className="flex items-center gap-1.5 flex-1 h-full rounded-none border-b-2 text-[11px] font-medium data-[active]:border-primary data-[active]:text-primary border-transparent text-muted-foreground"
          >
            <Activity className="size-3" />
            Events
          </TabsTrigger>
          <TabsTrigger
            value="terminal"
            className="flex items-center gap-1.5 flex-1 h-full rounded-none border-b-2 text-[11px] font-medium data-[active]:border-primary data-[active]:text-primary border-transparent text-muted-foreground"
          >
            <TerminalIcon className="size-3" />
            Terminal
          </TabsTrigger>
        </TabsList>

        <TabsContent value="chat" className="flex flex-col flex-1 min-h-0 mt-0">
          <ChatTab
            stagedAnnotations={stagedAnnotations}
            onSend={onSend ?? noopSend}
            onRemoveAnnotation={onRemoveAnnotation ?? noopRemove}
            onUploadFiles={onUploadFiles ?? noopUpload}
            disabled={disabled}
          />
        </TabsContent>

        <TabsContent value="events" className="flex flex-col flex-1 min-h-0 mt-0">
          <EventStreamPanel />
        </TabsContent>

        <TabsContent value="terminal" className="flex flex-col flex-1 min-h-0 mt-0">
          <TerminalTab
            workspaceId={activeWorkspaceId}
            visible={activeTab === 'terminal'}
          />
        </TabsContent>
      </Tabs>
    </div>
  )
}
