import { useNavigate, useRouterState } from '@tanstack/react-router'
import {
  Layers, Users, Zap, Briefcase, Settings,
  Search, Maximize, Minimize,
  type LucideIcon,
} from 'lucide-react'
import { useState, useCallback, useEffect } from 'react'
import { Button } from '@ship/primitives'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@ship/primitives'
import { Separator } from '@ship/primitives'
import { ThemeToggle } from '@ship/primitives'
import { useStudioShell } from './StudioShellContext'
import { useDaemon } from './hooks/useDaemon'

/** Views that get their own route and a spot on the rail. */
const RAIL_VIEWS: { to: string; icon: LucideIcon; label: string }[] = [
  { to: '/studio/session', icon: Layers, label: 'Session' },
  { to: '/studio/agents', icon: Users, label: 'Agents' },
  { to: '/studio/skills', icon: Zap, label: 'Skills' },
  { to: '/studio/jobs', icon: Briefcase, label: 'Jobs' },
]

export function StudioRail() {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const navigate = useNavigate()
  const { toggleCommandPalette } = useStudioShell()
  const { connected } = useDaemon()
  const [isFullscreen, setIsFullscreen] = useState(false)

  useEffect(() => {
    const onFs = () => setIsFullscreen(!!document.fullscreenElement)
    document.addEventListener('fullscreenchange', onFs)
    return () => document.removeEventListener('fullscreenchange', onFs)
  }, [])

  const handleFullscreen = useCallback(() => {
    if (document.fullscreenElement) void document.exitFullscreen()
    else void document.documentElement.requestFullscreen()
  }, [])

  return (
    <TooltipProvider>
      <nav className="w-12 shrink-0 bg-card border-r border-border/60 flex flex-col items-center py-2 gap-1">
        {/* Ship logo + connection */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="size-8 mb-1"
              onClick={() => void navigate({ to: '/studio/session' })}
            >
              <div className="relative">
                <img src="/ship-logos/ship_logo.svg" alt="" className="size-4" />
                <span className={`absolute -bottom-0.5 -right-0.5 w-1.5 h-1.5 rounded-full ${connected ? 'bg-status-green' : 'bg-status-red'}`} />
              </div>
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">
            {connected ? 'Connected' : 'Disconnected'}
          </TooltipContent>
        </Tooltip>

        <Separator className="w-6 my-1" />

        {/* View routes */}
        {RAIL_VIEWS.map((item) => {
          const isActive = pathname.startsWith(item.to)
          const Icon = item.icon
          return (
            <Tooltip key={item.to}>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className={`size-8 ${isActive ? 'bg-primary/10 text-primary' : ''}`}
                  onClick={() => void navigate({ to: item.to })}
                >
                  <Icon className="size-4" strokeWidth={isActive ? 2.2 : 1.8} />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="right">{item.label}</TooltipContent>
            </Tooltip>
          )
        })}

        <Separator className="w-6 my-1" />

        {/* Search = command palette */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="ghost" size="icon" className="size-8" onClick={toggleCommandPalette}>
              <Search className="size-4" strokeWidth={1.8} />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">Search (Cmd+K)</TooltipContent>
        </Tooltip>

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
            <Button variant="ghost" size="icon" className="size-8" onClick={handleFullscreen}>
              {isFullscreen
                ? <Minimize className="size-4" strokeWidth={1.8} />
                : <Maximize className="size-4" strokeWidth={1.8} />}
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
              className={`size-8 ${pathname.startsWith('/studio/settings') ? 'bg-primary/10 text-primary' : ''}`}
              onClick={() => void navigate({ to: '/studio/settings' })}
            >
              <Settings className="size-4" strokeWidth={pathname.startsWith('/studio/settings') ? 2.2 : 1.8} />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">Settings</TooltipContent>
        </Tooltip>
      </nav>
    </TooltipProvider>
  )
}
