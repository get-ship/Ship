import { Circle } from 'lucide-react'
import { Popover, PopoverContent, PopoverTrigger } from '@ship/primitives'
import { Separator } from '@ship/primitives'
import { useDaemon } from './hooks/useDaemon'

export function ProjectPicker() {
  const { workspaces, connected, agents } = useDaemon()
  const activeWs = workspaces.find((w) => String(w.status).toLowerCase() === 'active')

  return (
    <Popover>
      <PopoverTrigger
        render={
          <button className="w-8 h-8 rounded-lg flex items-center justify-center transition-colors hover:bg-muted">
            <div className="relative">
              <img src="/ship-logos/ship_logo.svg" alt="" className="size-4" />
              <span className={`absolute -bottom-0.5 -right-0.5 w-1.5 h-1.5 rounded-full ${connected ? 'bg-status-green' : 'bg-status-red'}`} />
            </div>
          </button>
        }
      />
      <PopoverContent side="right" align="start" className="w-72 p-0">
        {/* Connection */}
        <div className="px-3 py-2.5 flex items-center gap-2">
          <Circle className={`size-2 shrink-0 ${connected ? 'fill-emerald-500 text-emerald-500' : 'fill-red-500 text-red-500'}`} />
          <span className="text-xs font-medium">{connected ? 'Connected to daemon' : 'Disconnected'}</span>
        </div>

        <Separator />

        {/* Active workspace */}
        {activeWs ? (
          <div className="px-3 py-2.5">
            <div className="text-[10px] text-muted-foreground/60 uppercase tracking-wider mb-1">Active workspace</div>
            <div className="text-sm font-medium">{activeWs.branch}</div>
            {activeWs.active_agent && (
              <div className="text-xs text-muted-foreground mt-0.5">{activeWs.active_agent}</div>
            )}
          </div>
        ) : (
          <div className="px-3 py-2.5 text-xs text-muted-foreground">No active workspace</div>
        )}

        {/* Other workspaces */}
        {workspaces.length > 1 && (
          <>
            <Separator />
            <div className="px-3 py-2">
              <div className="text-[10px] text-muted-foreground/60 uppercase tracking-wider mb-1.5">Workspaces</div>
              <div className="space-y-0.5">
                {workspaces.filter((w) => w !== activeWs).map((ws) => (
                  <div key={ws.branch} className="flex items-center gap-2 py-1 text-xs text-muted-foreground">
                    <span className="size-1.5 rounded-full bg-muted-foreground/30 shrink-0" />
                    <span className="truncate">{ws.branch}</span>
                  </div>
                ))}
              </div>
            </div>
          </>
        )}

        {/* Agents */}
        {agents.length > 0 && (
          <>
            <Separator />
            <div className="px-3 py-2">
              <div className="text-[10px] text-muted-foreground/60 uppercase tracking-wider mb-1.5">Mesh agents</div>
              <div className="space-y-0.5">
                {agents.map((a) => (
                  <div key={a.agent_id} className="flex items-center gap-2 py-1 text-xs">
                    <span className={`size-1.5 rounded-full shrink-0 ${a.status === 'active' ? 'bg-emerald-500' : 'bg-muted-foreground/30'}`} />
                    <span className="truncate text-muted-foreground">{a.agent_id}</span>
                  </div>
                ))}
              </div>
            </div>
          </>
        )}
      </PopoverContent>
    </Popover>
  )
}
