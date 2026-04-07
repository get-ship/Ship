import { useState, useEffect } from 'react'
import { Users, Shield, Zap } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

// Three distinct agents with roles
const AGENTS: { icon: LucideIcon; name: string; role: string; color: string; bg: string }[] = [
  { icon: Zap, name: 'Implementer', role: 'Writes code', color: 'text-sky-400', bg: 'bg-sky-500/10' },
  { icon: Shield, name: 'Reviewer', role: 'Gates quality', color: 'text-amber-400', bg: 'bg-amber-500/10' },
  { icon: Users, name: 'Coordinator', role: 'Merges & deploys', color: 'text-emerald-400', bg: 'bg-emerald-500/10' },
]

// Slower pipeline with clear agent assignments
const PIPELINE_STAGES = [
  { id: 'dispatch', label: 'Human creates job in Studio', activeAgent: -1, from: 'human', to: 'runtime', duration: 2500 },
  { id: 'implement', label: 'Implementer picks up the job', activeAgent: 0, from: 'runtime', to: 'agent', duration: 3000 },
  { id: 'artifacts', label: 'Code written, artifacts staged', activeAgent: 0, from: 'agent', to: 'runtime', duration: 2500 },
  { id: 'review-notify', label: 'Human reviews artifacts in Studio', activeAgent: -1, from: 'runtime', to: 'human', duration: 3000 },
  { id: 'gate', label: 'Reviewer gates the changes', activeAgent: 1, from: 'runtime', to: 'agent', duration: 2500 },
  { id: 'approve', label: 'Gate passed', activeAgent: 1, from: 'agent', to: 'runtime', duration: 2000 },
  { id: 'merge', label: 'Coordinator merges to main', activeAgent: 2, from: 'runtime', to: 'agent', duration: 2500 },
  { id: 'done', label: 'Shipped', activeAgent: -1, from: 'runtime', to: 'runtime', duration: 3000 },
] as const

const RUNTIME_SERVICES = [
  { label: 'Event Kernel', color: 'text-primary' },
  { label: 'Supervisor', color: 'text-sky-400' },
  { label: 'Mesh', color: 'text-emerald-400' },
  { label: 'Pipelines', color: 'text-amber-400' },
]

const HUMAN_ACTIONS = ['Dispatch', 'Review', 'Annotate', 'Chat']

export function RuntimeDiagram() {
  const [stage, setStage] = useState(0)
  const [visible, setVisible] = useState(false)

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => { if (entry.isIntersecting) setVisible(true) },
      { threshold: 0.3 },
    )
    const el = document.getElementById('runtime-diagram')
    if (el) observer.observe(el)
    return () => observer.disconnect()
  }, [])

  useEffect(() => {
    if (!visible) return
    const current = PIPELINE_STAGES[stage]
    const timer = setTimeout(() => {
      setStage((s) => (s + 1) % PIPELINE_STAGES.length)
    }, current.duration)
    return () => clearTimeout(timer)
  }, [stage, visible])

  const current = PIPELINE_STAGES[stage]
  const flowLeft = current.from === 'agent' || (current.from === 'runtime' && current.to === 'human')
  const flowRight = current.from === 'human' || (current.from === 'runtime' && current.to === 'agent')

  return (
    <div id="runtime-diagram" className="relative mx-auto max-w-[52rem] px-4">
      {/* Status */}
      <div className="mb-8 flex items-center justify-center gap-2">
        <span className={`size-2 rounded-full transition-colors duration-500 ${
          current.id === 'done' ? 'bg-emerald-500' : 'bg-primary'
        } ${visible ? 'animate-pulse' : ''}`} />
        <span className="text-sm font-medium text-muted-foreground transition-all duration-500">
          {current.label}
        </span>
      </div>

      <div className="grid grid-cols-[1fr_2fr_1fr] gap-4 items-stretch">
        {/* Studio */}
        <div className={`rounded-xl border p-5 transition-all duration-700 ${
          current.from === 'human' || current.to === 'human'
            ? 'border-primary/30 bg-primary/[0.03] shadow-[0_0_30px_-10px] shadow-primary/10'
            : 'border-border/30 bg-card/20'
        }`}>
          <div className="text-[10px] font-semibold text-primary/60 uppercase tracking-wider mb-4">
            Studio
          </div>
          <div className="space-y-2">
            {HUMAN_ACTIONS.map((action, i) => (
              <div
                key={action}
                className={`flex items-center gap-2.5 rounded-lg px-2.5 py-2 text-xs transition-all duration-500 ${
                  (current.id === 'dispatch' && i === 0) ||
                  (current.id === 'review-notify' && (i === 1 || i === 2)) ||
                  ((current.id === 'implement' || current.id === 'gate') && i === 3)
                    ? 'bg-primary/10 text-primary font-medium'
                    : 'text-muted-foreground/40'
                }`}
              >
                <span className={`size-2 rounded-full transition-all duration-500 ${
                  (current.id === 'dispatch' && i === 0) ||
                  (current.id === 'review-notify' && (i === 1 || i === 2)) ||
                  ((current.id === 'implement' || current.id === 'gate') && i === 3)
                    ? 'bg-primary scale-125'
                    : 'bg-muted-foreground/15'
                }`} />
                {action}
              </div>
            ))}
          </div>
        </div>

        {/* Runtime */}
        <div className="rounded-xl border border-border/40 bg-card/30 p-5">
          <div className="text-[10px] font-semibold text-muted-foreground/50 uppercase tracking-wider mb-4 text-center">
            Ship Runtime
          </div>

          <div className="grid grid-cols-2 gap-2 mb-5">
            {RUNTIME_SERVICES.map((svc) => {
              const active =
                svc.label === 'Event Kernel' ||
                (svc.label === 'Supervisor' && (current.id === 'dispatch' || current.id === 'implement')) ||
                (svc.label === 'Pipelines' && (current.id === 'gate' || current.id === 'merge' || current.id === 'approve')) ||
                (svc.label === 'Mesh' && (current.id === 'implement' || current.id === 'artifacts'))
              return (
                <div
                  key={svc.label}
                  className={`rounded-lg border px-3 py-2.5 text-center transition-all duration-500 ${
                    active
                      ? 'border-border/50 bg-background/60'
                      : 'border-border/20 bg-background/20'
                  }`}
                >
                  <span className={`text-[10px] font-medium transition-opacity duration-500 ${svc.color} ${active ? 'opacity-100' : 'opacity-30'}`}>
                    {svc.label}
                  </span>
                </div>
              )
            })}
          </div>

          {/* Flow lines */}
          <div className="flex items-center justify-between px-3">
            <FlowDots active={flowLeft} direction="left" />
            <span className={`text-[9px] font-medium transition-all duration-500 ${
              current.id === 'done' ? 'text-emerald-400' : 'text-muted-foreground/30'
            }`}>
              {current.id === 'done' ? 'shipped' : 'routing'}
            </span>
            <FlowDots active={flowRight} direction="right" />
          </div>
        </div>

        {/* Agents */}
        <div className={`rounded-xl border p-5 transition-all duration-700 ${
          current.from === 'agent' || current.to === 'agent'
            ? 'border-sky-500/30 bg-sky-500/[0.03] shadow-[0_0_30px_-10px] shadow-sky-500/10'
            : 'border-border/30 bg-card/20'
        }`}>
          <div className="text-[10px] font-semibold text-sky-400/60 uppercase tracking-wider mb-4">
            Agents
          </div>
          <div className="space-y-2">
            {AGENTS.map((agent, i) => {
              const Icon = agent.icon
              const active = current.activeAgent === i
              return (
                <div
                  key={agent.name}
                  className={`flex items-center gap-2.5 rounded-lg px-2.5 py-2 transition-all duration-500 ${
                    active ? `${agent.bg} font-medium` : ''
                  }`}
                >
                  <div className={`size-6 rounded-md flex items-center justify-center transition-all duration-500 ${
                    active ? agent.bg : 'bg-muted-foreground/5'
                  }`}>
                    <Icon className={`size-3 transition-all duration-500 ${
                      active ? agent.color : 'text-muted-foreground/20'
                    }`} />
                  </div>
                  <div>
                    <div className={`text-xs transition-all duration-500 ${
                      active ? agent.color : 'text-muted-foreground/40'
                    }`}>
                      {agent.name}
                    </div>
                    <div className={`text-[9px] transition-all duration-500 ${
                      active ? 'text-muted-foreground/60' : 'text-muted-foreground/20'
                    }`}>
                      {agent.role}
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      </div>

      {/* Pipeline progress */}
      <div className="mt-8 flex items-center justify-center gap-1.5">
        {PIPELINE_STAGES.map((s, i) => (
          <div
            key={s.id}
            className={`h-1 rounded-full transition-all duration-500 ${
              i < stage ? 'w-6 bg-emerald-500/50'
                : i === stage ? 'w-10 bg-primary'
                : 'w-4 bg-muted-foreground/10'
            }`}
          />
        ))}
      </div>
    </div>
  )
}

function FlowDots({ active, direction }: { active: boolean; direction: 'left' | 'right' }) {
  return (
    <div className={`flex items-center gap-1 transition-all duration-500 ${active ? 'opacity-100' : 'opacity-10'}`}>
      {direction === 'left' && <span className="text-[9px] text-primary/70">◄</span>}
      {[0, 1, 2].map((i) => (
        <span
          key={i}
          className={`size-1 rounded-full transition-all duration-300 ${active ? 'bg-primary' : 'bg-muted-foreground/20'}`}
          style={active ? { animation: 'pulse 1.5s ease-in-out infinite', animationDelay: `${i * 200}ms` } : undefined}
        />
      ))}
      {direction === 'right' && <span className="text-[9px] text-primary/70">►</span>}
    </div>
  )
}
