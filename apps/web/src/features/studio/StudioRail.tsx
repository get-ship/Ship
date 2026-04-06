import {
  FolderTree, GitBranch, LayoutGrid, Briefcase,
  Activity, MessageSquare, Settings,
  type LucideIcon,
} from 'lucide-react'
import { Button } from '@ship/primitives'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@ship/primitives'
import { Separator } from '@ship/primitives'
import { ThemeToggle } from '@ship/primitives'
import { useStudioShell, type PanelId } from './StudioShellContext'
import { useDaemon } from './hooks/useDaemon'

interface RailEntry {
  id: PanelId
  icon: LucideIcon
  label: string
  kbd?: string
}

const PANEL_ITEMS: RailEntry[] = [
  { id: 'files', icon: FolderTree, label: 'Files', kbd: 'Cmd+B' },
  { id: 'git', icon: GitBranch, label: 'Git' },
  { id: 'views', icon: LayoutGrid, label: 'Views' },
  { id: 'jobs', icon: Briefcase, label: 'Jobs', kbd: 'Cmd+Shift+J' },
]

export function StudioRail() {
  const { activePanel, togglePanel, eventsOpen, toggleEvents, chatOpen, toggleChat } = useStudioShell()
  const { connected } = useDaemon()

  return (
    <TooltipProvider>
      <nav className="w-12 shrink-0 bg-card border-r border-border/60 flex flex-col items-center py-2 gap-1">
        {/* Project logo + connection status */}
        <Tooltip>
          <TooltipTrigger asChild>
            <button className="w-8 h-8 rounded-lg flex items-center justify-center mb-1 transition-colors hover:bg-muted">
              <div className="relative">
                <img src="/ship-logos/ship_logo.svg" alt="" className="size-4" />
                <span className={`absolute -bottom-0.5 -right-0.5 w-1.5 h-1.5 rounded-full ${connected ? 'bg-status-green' : 'bg-status-red'}`} />
              </div>
            </button>
          </TooltipTrigger>
          <TooltipContent side="right">
            {connected ? 'Connected to daemon' : 'Daemon disconnected'}
          </TooltipContent>
        </Tooltip>

        <Separator className="w-6 my-1" />

        {/* Panel toggles — each opens an overlay */}
        {PANEL_ITEMS.map((entry) => (
          <Tooltip key={entry.id}>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className={`size-8 ${activePanel === entry.id ? 'bg-primary/10 text-primary' : ''}`}
                onClick={() => togglePanel(entry.id)}
              >
                <entry.icon className="size-4" strokeWidth={activePanel === entry.id ? 2.2 : 1.8} />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="right">
              {entry.label}{entry.kbd ? ` (${entry.kbd})` : ''}
            </TooltipContent>
          </Tooltip>
        ))}

        <div className="flex-1" />

        {/* Floating toggles */}
        <Separator className="w-6 mb-1" />

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className={`size-8 ${eventsOpen ? 'bg-primary/10 text-primary' : ''}`}
              onClick={toggleEvents}
            >
              <Activity className="size-4" strokeWidth={eventsOpen ? 2.2 : 1.8} />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">Events</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className={`size-8 ${chatOpen ? 'bg-primary/10 text-primary' : ''}`}
              onClick={toggleChat}
            >
              <MessageSquare className="size-4" strokeWidth={chatOpen ? 2.2 : 1.8} />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">Agent Chat</TooltipContent>
        </Tooltip>

        <Separator className="w-6 my-1" />

        {/* Theme + Settings */}
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="size-8 flex items-center justify-center">
              <ThemeToggle variant="icon" />
            </div>
          </TooltipTrigger>
          <TooltipContent side="right">Theme</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="ghost" size="icon" className="size-8">
              <Settings className="size-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">Settings</TooltipContent>
        </Tooltip>
      </nav>
    </TooltipProvider>
  )
}
