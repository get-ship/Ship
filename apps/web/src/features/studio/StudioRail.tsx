import {
  FolderTree, Search, Briefcase, Activity, MessageSquare, Settings,
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
}

const PANELS: RailEntry[] = [
  { id: 'artifacts', icon: FolderTree, label: 'Artifacts' },
  { id: 'search', icon: Search, label: 'Search' },
  { id: 'jobs', icon: Briefcase, label: 'Jobs' },
]

export function StudioRail() {
  const { activePanel, setActivePanel, toggleEvents, toggleChat, eventsOpen, chatOpen } = useStudioShell()
  const { connected, workspaces } = useDaemon()
  const activeWs = workspaces.find((w) => String(w.status).toLowerCase() === 'active')

  return (
    <TooltipProvider>
      <nav className="w-12 shrink-0 bg-card border-r border-border/60 flex flex-col items-center py-2 gap-1">
        {/* Workspace indicator */}
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              className="w-8 h-8 rounded-lg flex items-center justify-center mb-1 transition-colors hover:bg-muted"
              onClick={() => setActivePanel(activePanel === 'workspaces' ? null : 'workspaces')}
            >
              <div className="relative">
                <img src="/ship-logos/ship_logo.svg" alt="" className="size-4" />
                <span className={`absolute -bottom-0.5 -right-0.5 w-1.5 h-1.5 rounded-full ${connected ? 'bg-status-green' : 'bg-status-red'}`} />
              </div>
            </button>
          </TooltipTrigger>
          <TooltipContent side="right">
            {activeWs ? activeWs.branch : 'No workspace'}
          </TooltipContent>
        </Tooltip>

        <Separator className="w-6 my-1" />

        {/* Panel toggles */}
        {PANELS.map((entry) => (
          <RailButton
            key={entry.id}
            icon={entry.icon}
            label={entry.label}
            active={activePanel === entry.id}
            onClick={() => setActivePanel(activePanel === entry.id ? null : entry.id)}
          />
        ))}

        <div className="flex-1" />

        {/* Floating panel toggles */}
        <Separator className="w-6 mb-1" />
        <RailButton
          icon={Activity}
          label="Events"
          active={eventsOpen}
          onClick={toggleEvents}
        />
        <RailButton
          icon={MessageSquare}
          label="Chat"
          active={chatOpen}
          onClick={toggleChat}
        />

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
        <RailButton
          icon={Settings}
          label="Settings"
          active={activePanel === 'settings'}
          onClick={() => setActivePanel(activePanel === 'settings' ? null : 'settings')}
        />
      </nav>
    </TooltipProvider>
  )
}

function RailButton({
  icon: Icon,
  label,
  active,
  onClick,
}: {
  icon: LucideIcon
  label: string
  active: boolean
  onClick: () => void
}) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className={`size-8 ${active ? 'bg-primary/10 text-primary' : ''}`}
          onClick={onClick}
        >
          <Icon className="size-4" strokeWidth={active ? 2.2 : 1.8} />
        </Button>
      </TooltipTrigger>
      <TooltipContent side="right">{label}</TooltipContent>
    </Tooltip>
  )
}
