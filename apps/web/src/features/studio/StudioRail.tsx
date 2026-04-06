import { useNavigate, useRouterState } from '@tanstack/react-router'
import { Users, Zap, Layers, Settings, PanelLeft } from 'lucide-react'
import { Button } from '@ship/primitives'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@ship/primitives'
import { Separator } from '@ship/primitives'
import { useStudioShell } from './StudioShellContext'

const NAV_ITEMS = [
  { to: '/studio/session', icon: Layers, label: 'Session' },
  { to: '/studio/agents', icon: Users, label: 'Agents' },
  { to: '/studio/skills', icon: Zap, label: 'Skills' },
] as const

const BOTTOM_ITEMS = [
  { to: '/studio/settings', icon: Settings, label: 'Settings' },
] as const

export function StudioRail() {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const navigate = useNavigate()
  const { sidebarOpen, toggleSidebar } = useStudioShell()

  return (
    <TooltipProvider>
      <nav className="w-12 shrink-0 bg-card border-r border-border/60 flex flex-col items-center py-2 gap-1">
        {/* Sidebar toggle */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className={`size-8 mb-1 ${sidebarOpen ? 'bg-primary/10 text-primary' : ''}`}
              onClick={toggleSidebar}
            >
              <PanelLeft className="size-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">Toggle sidebar</TooltipContent>
        </Tooltip>

        <Separator className="w-6 my-1" />

        {/* Main nav */}
        <div className="flex flex-col items-center gap-1">
          {NAV_ITEMS.map((item) => {
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
        </div>

        {/* Spacer */}
        <div className="flex-1" />

        {/* Bottom */}
        <Separator className="w-6 mb-1" />
        {BOTTOM_ITEMS.map((item) => {
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
      </nav>
    </TooltipProvider>
  )
}
