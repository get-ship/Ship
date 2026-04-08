import { useNavigate } from '@tanstack/react-router'
import { Button } from '@ship/primitives'
import { Tooltip, TooltipContent, TooltipTrigger } from '@ship/primitives'
import { useDaemon } from './hooks/useDaemonData'

export function ProjectPicker() {
  const { connected, workspaces } = useDaemon()
  const navigate = useNavigate()
  const activeWs = workspaces.find((w) => String(w.status).toLowerCase() === 'active')

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="size-8"
          onClick={() => void navigate({ to: '/studio' })}
        >
          <div className="relative">
            <img src="/ship-logos/ship_logo.svg" alt="" className="size-4" />
            <span className={`absolute -bottom-0.5 -right-0.5 w-1.5 h-1.5 rounded-full ${connected ? 'bg-status-green' : 'bg-status-red'}`} />
          </div>
        </Button>
      </TooltipTrigger>
      <TooltipContent side="right">
        <div className="text-xs">
          <div className="font-medium">{connected ? 'Ship Studio' : 'Disconnected'}</div>
          {activeWs && <div className="text-muted-foreground">{activeWs.branch}</div>}
        </div>
      </TooltipContent>
    </Tooltip>
  )
}
