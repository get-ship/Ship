import { type LucideIcon, Briefcase, FileText, Search, Layers, Activity, MessageSquare, Settings, GitBranch } from 'lucide-react'

export type ViewId =
  | 'jobs'
  | 'artifacts'
  | 'agent-editor'
  | 'skills-ide'
  | 'settings'
  | 'sprint-dag'

interface RailProps {
  activeView: ViewId
  onViewChange: (view: ViewId) => void
  onTogglePanel: () => void
  onToggleEvents: () => void
  onToggleChat: () => void
  workspaceBranch: string | null
}

interface RailItem {
  id: ViewId | 'panel' | 'events' | 'chat'
  icon: LucideIcon
  label: string
  type: 'view' | 'action'
  separator?: boolean
}

const RAIL_ITEMS: RailItem[] = [
  { id: 'jobs', icon: Briefcase, label: 'Jobs', type: 'view' },
  { id: 'artifacts', icon: FileText, label: 'Artifacts', type: 'view' },
  { id: 'agent-editor', icon: Layers, label: 'Agent Editor', type: 'view' },
  { id: 'skills-ide', icon: Search, label: 'Skills IDE', type: 'view' },
  { id: 'settings', icon: Settings, label: 'Settings', type: 'view', separator: true },
  { id: 'events', icon: Activity, label: 'Events', type: 'action', separator: true },
  { id: 'chat', icon: MessageSquare, label: 'Chat', type: 'action' },
]

export function Rail({ activeView, onViewChange, onTogglePanel, onToggleEvents, onToggleChat, workspaceBranch }: RailProps) {
  return (
    <nav className="w-12 shrink-0 bg-card border-r border-border/60 flex flex-col items-center py-2 gap-1">
      {/* Workspace indicator */}
      <button
        onClick={onTogglePanel}
        className="w-8 h-8 rounded-lg flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-muted transition-colors mb-2"
        title={workspaceBranch ?? 'No workspace'}
      >
        <GitBranch className="size-4" />
      </button>

      <div className="w-6 border-t border-border/40 mb-1" />

      {/* Main items */}
      <div className="flex-1 flex flex-col items-center gap-1">
        {RAIL_ITEMS.map((item) => {
          if (item.separator && item.type !== 'action') {
            return (
              <div key={`sep-${item.id}`} className="flex flex-col items-center gap-1">
                <div className="w-6 border-t border-border/40 my-1" />
                <RailButton
                  item={item}
                  active={item.type === 'view' && activeView === item.id}
                  onClick={() => {
                    if (item.type === 'view') onViewChange(item.id as ViewId)
                  }}
                />
              </div>
            )
          }
          if (item.type === 'action') return null
          return (
            <RailButton
              key={item.id}
              item={item}
              active={activeView === item.id}
              onClick={() => onViewChange(item.id as ViewId)}
            />
          )
        })}
      </div>

      {/* Bottom actions */}
      <div className="flex flex-col items-center gap-1 mt-auto">
        <div className="w-6 border-t border-border/40 mb-1" />
        <RailButton
          item={{ id: 'events', icon: Activity, label: 'Events', type: 'action' }}
          active={false}
          onClick={onToggleEvents}
        />
        <RailButton
          item={{ id: 'chat', icon: MessageSquare, label: 'Chat', type: 'action' }}
          active={false}
          onClick={onToggleChat}
        />
      </div>
    </nav>
  )
}

function RailButton({ item, active, onClick }: { item: RailItem; active: boolean; onClick: () => void }) {
  const Icon = item.icon
  return (
    <button
      onClick={onClick}
      title={item.label}
      className={`w-8 h-8 rounded-lg flex items-center justify-center transition-colors ${
        active
          ? 'bg-primary/10 text-primary'
          : 'text-muted-foreground hover:text-foreground hover:bg-muted'
      }`}
    >
      <Icon className="size-4" />
    </button>
  )
}
