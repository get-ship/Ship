import { useState, useEffect } from 'react'
import { useNavigate, useRouterState } from '@tanstack/react-router'
import {
  Layers, Users, Zap, LayoutList, GitBranch, GitMerge,
  Search, Settings, MessageSquare, Sun, Moon,
  type LucideIcon,
} from 'lucide-react'
import { Button } from '@ship/primitives'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@ship/primitives'
import { Separator } from '@ship/primitives'
import { useStudioShell } from './StudioShellContext'
import { ProjectPicker } from './ProjectPicker'
import { ViewsOverlay } from './ViewsOverlay'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'

const PINNED_VIEWS: { to: string; icon: LucideIcon; label: string }[] = [
  { to: '/studio/session', icon: Layers, label: 'Artifacts' },
  { to: '/studio/git', icon: GitBranch, label: 'Git' },
  { to: '/studio/agents', icon: Users, label: 'Agents' },
  { to: '/studio/skills', icon: Zap, label: 'Skills' },
  { to: '/studio/jobs', icon: LayoutList, label: 'Jobs' },
  { to: '/studio/pipelines', icon: GitMerge, label: 'Pipelines' },
]

function useTheme() {
  const [isDark, setIsDark] = useState(true)

  useEffect(() => {
    const stored = window.localStorage.getItem('theme')
    if (stored === 'light' || stored === 'dark') {
      setIsDark(stored === 'dark')
    } else {
      setIsDark(window.matchMedia('(prefers-color-scheme: dark)').matches)
    }
  }, [])

  const toggle = () => {
    const next = !isDark
    setIsDark(next)
    const mode = next ? 'dark' : 'light'
    document.documentElement.classList.remove('light', 'dark')
    document.documentElement.classList.add(mode)
    document.documentElement.setAttribute('data-theme', mode)
    document.documentElement.style.colorScheme = mode
    window.localStorage.setItem('theme', mode)
  }

  return { isDark, toggle }
}

export function StudioRail() {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const navigate = useNavigate()
  const { toggleCommandPalette, drawerOpen, toggleDrawer } = useStudioShell()
  const { jobs } = useDaemon()
  const { isDark, toggle: toggleTheme } = useTheme()

  const runningJobs = jobs.filter((j) => j.status === 'dispatched' || j.status === 'pending').length

  return (
    <TooltipProvider>
      <nav className="w-12 shrink-0 border-r border-border/60 flex flex-col items-center py-2 gap-1 text-muted-foreground">
        <ProjectPicker />

        <Separator className="w-6 my-1" />

        {PINNED_VIEWS.map((item) => {
          const isActive = pathname.startsWith(item.to)
          const Icon = item.icon
          const isJobs = item.to === '/studio/jobs'
          return (
            <Tooltip key={item.to}>
              <TooltipTrigger asChild>
                <div className="relative">
                  <Button
                    variant="ghost"
                    size="icon"
                    className={`size-8 ${isActive ? 'bg-primary/10 text-primary' : ''}`}
                    onClick={() => void navigate({ to: item.to })}
                  >
                    <Icon className="size-4" strokeWidth={isActive ? 2.2 : 1.8} />
                  </Button>
                  {isJobs && runningJobs > 0 && (
                    <span className="absolute -top-0.5 -right-0.5 flex size-3.5 items-center justify-center rounded-full bg-sky-500 text-[8px] font-bold text-white leading-none">
                      {runningJobs > 9 ? '9+' : runningJobs}
                    </span>
                  )}
                </div>
              </TooltipTrigger>
              <TooltipContent side="right">{item.label}</TooltipContent>
            </Tooltip>
          )
        })}

        <Separator className="w-6 my-1" />

        <ViewsOverlay />

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

        {/* Agent drawer toggle */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className={`size-8 ${drawerOpen ? 'bg-primary/10 text-primary' : ''}`}
              onClick={toggleDrawer}
            >
              <MessageSquare className="size-4" strokeWidth={drawerOpen ? 2.2 : 1.8} />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">Agent Panel (Cmd+/)</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="ghost" size="icon" className="size-8" onClick={toggleTheme}>
              {isDark
                ? <Sun className="size-4" strokeWidth={1.8} />
                : <Moon className="size-4" strokeWidth={1.8} />}
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">{isDark ? 'Light mode' : 'Dark mode'}</TooltipContent>
        </Tooltip>

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
