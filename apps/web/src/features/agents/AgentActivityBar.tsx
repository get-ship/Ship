import {
  Zap,
  Grid3X3,
  Shield,
  FileText,
  SlidersHorizontal,
} from 'lucide-react'

export const SECTION_DEFS = [
  { id: 'skills', label: 'Skills', icon: Zap, countable: true },
  { id: 'mcp', label: 'MCP Servers', icon: Grid3X3, countable: true },
  { id: 'permissions', label: 'Permissions', icon: Shield, countable: false },
  { id: 'rules', label: 'Rules', icon: FileText, countable: true },
  { id: 'providers', label: 'Providers', icon: SlidersHorizontal, countable: true },
] as const

interface AgentActivityBarProps {
  activeSection: string
  onSectionClick: (sectionId: string) => void
  counts: Record<string, number>
}

export function AgentActivityBar({
  activeSection,
  onSectionClick,
  counts,
}: AgentActivityBarProps) {
  return (
    <nav
      aria-label="Agent sections"
      className="flex items-center gap-1 px-4 py-1.5 border-b border-border/60 shrink-0 overflow-x-auto"
    >
      {SECTION_DEFS.map((section) => {
        const Icon = section.icon
        const isActive = activeSection === section.id
        const count = section.countable ? counts[section.id] : undefined

        return (
          <button
            key={section.id}
            onClick={() => onSectionClick(section.id)}
            className={`flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs font-medium whitespace-nowrap transition-colors ${
              isActive
                ? 'bg-primary/10 text-primary'
                : 'text-muted-foreground hover:text-foreground hover:bg-muted/40'
            }`}
          >
            <Icon className="size-3.5" strokeWidth={isActive ? 2.2 : 1.6} />
            {section.label}
            {count !== undefined && count > 0 && (
              <span className="text-[10px] opacity-60 tabular-nums">{count}</span>
            )}
          </button>
        )
      })}
    </nav>
  )
}
