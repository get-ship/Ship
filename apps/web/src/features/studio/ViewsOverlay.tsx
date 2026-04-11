import { useNavigate, useRouterState } from '@tanstack/react-router'
import { Layers, Users, Zap, Briefcase, GitBranch, Settings, LayoutGrid, MonitorPlay } from 'lucide-react'
import { Popover, PopoverContent, PopoverTrigger } from '@ship/primitives'
import { Separator } from '@ship/primitives'
import { useViews } from './hooks/useDaemonData'
import type { LucideIcon } from 'lucide-react'

interface ViewDef {
  to: string
  icon: LucideIcon
  label: string
  scope: 'project' | 'workspace'
}

const BUILT_IN_VIEWS: ViewDef[] = [
  { to: '/studio', icon: LayoutGrid, label: 'Dashboard', scope: 'project' },
  { to: '/studio/session', icon: Layers, label: 'Session', scope: 'workspace' },
  { to: '/studio/git', icon: GitBranch, label: 'Git', scope: 'workspace' },
  { to: '/studio/agents', icon: Users, label: 'Agents', scope: 'project' },
  { to: '/studio/skills', icon: Zap, label: 'Skills', scope: 'project' },
  { to: '/studio/workflows', icon: Briefcase, label: 'Workflows', scope: 'project' },
  { to: '/studio/settings', icon: Settings, label: 'Settings', scope: 'project' },
]

export function ViewsOverlay() {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const navigate = useNavigate()
  const { data: runtimeViews } = useViews()

  const projectViews = BUILT_IN_VIEWS.filter((v) => v.scope === 'project')
  const workspaceViews = BUILT_IN_VIEWS.filter((v) => v.scope === 'workspace')

  return (
    <Popover>
      <PopoverTrigger
        render={
          <button className="w-8 h-8 rounded-lg flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-muted transition-colors">
            <LayoutGrid className="size-4" strokeWidth={1.8} />
          </button>
        }
      />
      <PopoverContent side="right" align="start" className="w-56 p-0">
        {/* Workspace views */}
        <div className="px-2 py-2">
          <div className="text-[10px] text-muted-foreground/60 uppercase tracking-wider px-2 mb-1">Workspace</div>
          {workspaceViews.map((v) => (
            <ViewItem
              key={v.to}
              view={v}
              active={isActive(pathname, v.to)}
              onClick={() => void navigate({ to: v.to })}
            />
          ))}
        </div>

        <Separator />

        {/* Project views */}
        <div className="px-2 py-2">
          <div className="text-[10px] text-muted-foreground/60 uppercase tracking-wider px-2 mb-1">Project</div>
          {projectViews.map((v) => (
            <ViewItem
              key={v.to}
              view={v}
              active={isActive(pathname, v.to)}
              onClick={() => void navigate({ to: v.to })}
            />
          ))}
        </div>

        {/* Runtime views */}
        {runtimeViews && runtimeViews.length > 0 && (
          <>
            <Separator />
            <div className="px-2 py-2">
              <div className="text-[10px] text-muted-foreground/60 uppercase tracking-wider px-2 mb-1">Extensions</div>
              {runtimeViews.map((v) => {
                const viewTo = `/studio/view/${encodeURIComponent(v.name)}`
                return (
                  <ViewItem
                    key={v.name}
                    view={{ to: viewTo, icon: MonitorPlay, label: v.name, scope: 'workspace' }}
                    active={pathname.startsWith(viewTo)}
                    onClick={() => void navigate({ to: viewTo })}
                  />
                )
              })}
            </div>
          </>
        )}
      </PopoverContent>
    </Popover>
  )
}

function ViewItem({ view, active, onClick }: { view: ViewDef; active: boolean; onClick: () => void }) {
  const Icon = view.icon
  return (
    <button
      onClick={onClick}
      className={`w-full flex items-center gap-2.5 px-2 py-1.5 rounded-md text-xs transition-colors ${
        active
          ? 'bg-primary/10 text-primary font-medium'
          : 'text-muted-foreground hover:bg-muted hover:text-foreground'
      }`}
    >
      <Icon className="size-3.5 shrink-0" />
      {view.label}
    </button>
  )
}

function isActive(pathname: string, to: string): boolean {
  if (to === '/studio') return pathname === '/studio' || pathname === '/studio/'
  return pathname.startsWith(to)
}
