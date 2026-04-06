import { useNavigate } from '@tanstack/react-router'
import {
  FolderTree, GitBranch, LayoutGrid, Briefcase,
  Search, Maximize, Minimize, Settings,
  type LucideIcon,
} from 'lucide-react'
import { useState, useCallback, useEffect } from 'react'
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
  { id: 'jobs', icon: Briefcase, label: 'Jobs' },
]

export function StudioRail() {
  const { activePanel, togglePanel, toggleCommandPalette } = useStudioShell()
  const { connected } = useDaemon()
  const navigate = useNavigate()
  const [isFullscreen, setIsFullscreen] = useState(false)

  useEffect(() => {
    function onFsChange() {
      setIsFullscreen(!!document.fullscreenElement)
    }
    document.addEventListener('fullscreenchange', onFsChange)
    return () => document.removeEventListener('fullscreenchange', onFsChange)
  }, [])

  const handleFullscreen = useCallback(() => {
    if (document.fullscreenElement) {
      void document.exitFullscreen()
    } else {
      void document.documentElement.requestFullscreen()
    }
  }, [])

  return (
    <TooltipProvider>
      <nav className="w-12 shrink-0 bg-card border-r border-border/60 flex flex-col items-center py-2 gap-1">
        {/* Ship logo + connection dot */}
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

        {/* Panel toggles */}
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

        <Separator className="w-6 my-1" />

        {/* Search = command palette */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="size-8"
              onClick={toggleCommandPalette}
            >
              <Search className="size-4" strokeWidth={1.8} />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">Search (Cmd+K)</TooltipContent>
        </Tooltip>

        {/* Spacer */}
        <div className="flex-1" />

        <Separator className="w-6 mb-1" />

        {/* Theme */}
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="size-8 flex items-center justify-center">
              <ThemeToggle variant="icon" />
            </div>
          </TooltipTrigger>
          <TooltipContent side="right">Theme</TooltipContent>
        </Tooltip>

        {/* Fullscreen */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="size-8"
              onClick={handleFullscreen}
            >
              {isFullscreen
                ? <Minimize className="size-4" strokeWidth={1.8} />
                : <Maximize className="size-4" strokeWidth={1.8} />
              }
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">
            {isFullscreen ? 'Exit fullscreen (F11)' : 'Fullscreen (F11)'}
          </TooltipContent>
        </Tooltip>

        {/* Settings */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="size-8"
              onClick={() => void navigate({ to: '/studio/settings' })}
            >
              <Settings className="size-4" strokeWidth={1.8} />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">Settings</TooltipContent>
        </Tooltip>
      </nav>
    </TooltipProvider>
  )
}
