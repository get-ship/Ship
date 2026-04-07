import { useState, useEffect } from 'react'

// Pipeline stages that animate through the diagram
const PIPELINE_STAGES = [
  { id: 'dispatch', label: 'Job created', from: 'human', to: 'runtime', duration: 1500 },
  { id: 'assign', label: 'Agent dispatched', from: 'runtime', to: 'agent', duration: 1500 },
  { id: 'work', label: 'Agent working...', from: 'agent', to: 'runtime', duration: 2000 },
  { id: 'artifact', label: 'Artifacts ready', from: 'runtime', to: 'human', duration: 1500 },
  { id: 'review', label: 'Human reviews', from: 'human', to: 'runtime', duration: 1500 },
  { id: 'gate', label: 'Gate passed', from: 'runtime', to: 'agent', duration: 1200 },
  { id: 'merge', label: 'Merged', from: 'runtime', to: 'runtime', duration: 1200 },
] as const

const RUNTIME_SERVICES = [
  { label: 'Event Kernel', color: 'text-primary' },
  { label: 'Supervisor', color: 'text-sky-400' },
  { label: 'Mesh', color: 'text-emerald-400' },
  { label: 'Pipelines', color: 'text-amber-400' },
]

const AGENTS = [
  { name: 'claude', label: 'Claude Code' },
  { name: 'cursor', label: 'Cursor' },
  { name: 'codex', label: 'Codex CLI' },
  { name: 'gemini', label: 'Gemini CLI' },
]

const HUMAN_ACTIONS = [
  'Review',
  'Annotate',
  'Dispatch',
  'Chat',
]

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
  const flowInternal = current.from === 'runtime' && current.to === 'runtime'

  return (
    <div id="runtime-diagram" className="relative mx-auto max-w-[52rem] px-4">
      {/* Status bar */}
      <div className="mb-6 flex items-center justify-center gap-2">
        <span className={`size-2 rounded-full transition-colors duration-300 ${
          stage === PIPELINE_STAGES.length - 1 ? 'bg-emerald-500' : 'bg-primary'
        } ${visible ? 'animate-pulse' : ''}`} />
        <span className="text-xs font-medium text-muted-foreground transition-all duration-300">
          {current.label}
        </span>
      </div>

      <div className="grid grid-cols-[1fr_2fr_1fr] gap-3 items-stretch">
        {/* Human / Studio side */}
        <div className={`rounded-xl border bg-card/40 p-4 transition-all duration-500 ${
          current.from === 'human' || current.to === 'human'
            ? 'border-primary/40 shadow-[0_0_20px_-5px] shadow-primary/10'
            : 'border-border/40'
        }`}>
          <div className="text-[10px] font-semibold text-primary/70 uppercase tracking-wider mb-3">
            Ship Studio
          </div>
          <div className="space-y-1.5">
            {HUMAN_ACTIONS.map((action, i) => (
              <div
                key={action}
                className={`flex items-center gap-2 rounded-md px-2 py-1.5 text-[11px] transition-all duration-300 ${
                  (current.id === 'dispatch' && i === 2) ||
                  (current.id === 'review' && i === 0) ||
                  (current.id === 'artifact' && i === 1) ||
                  ((current.id === 'work' || current.id === 'assign') && i === 3)
                    ? 'bg-primary/10 text-primary font-medium'
                    : 'text-muted-foreground/50'
                }`}
              >
                <span className={`size-1.5 rounded-full transition-colors duration-300 ${
                  (current.id === 'dispatch' && i === 2) ||
                  (current.id === 'review' && i === 0) ||
                  (current.id === 'artifact' && i === 1) ||
                  ((current.id === 'work' || current.id === 'assign') && i === 3)
                    ? 'bg-primary'
                    : 'bg-muted-foreground/20'
                }`} />
                {action}
              </div>
            ))}
          </div>
        </div>

        {/* Runtime center */}
        <div className={`rounded-xl border bg-card/60 p-4 transition-all duration-500 ${
          current.from === 'runtime' || current.to === 'runtime'
            ? 'border-border/60'
            : 'border-border/30'
        }`}>
          <div className="text-[10px] font-semibold text-muted-foreground/70 uppercase tracking-wider mb-3 text-center">
            Ship Runtime
          </div>

          {/* Services grid */}
          <div className="grid grid-cols-2 gap-2 mb-4">
            {RUNTIME_SERVICES.map((svc) => (
              <div
                key={svc.label}
                className={`rounded-lg border border-border/30 bg-background/40 px-3 py-2 text-center transition-all duration-300 ${
                  (svc.label === 'Event Kernel') ||
                  (svc.label === 'Supervisor' && (current.id === 'dispatch' || current.id === 'assign')) ||
                  (svc.label === 'Pipelines' && (current.id === 'gate' || current.id === 'merge')) ||
                  (svc.label === 'Mesh' && (current.id === 'work' || current.id === 'assign'))
                    ? 'border-border/60 shadow-sm'
                    : ''
                }`}
              >
                <span className={`text-[10px] font-medium ${svc.color}`}>
                  {svc.label}
                </span>
              </div>
            ))}
          </div>

          {/* Flow indicators */}
          <div className="flex items-center justify-between px-2">
            <FlowArrow active={flowLeft} direction="left" />
            <div className={`flex items-center gap-1 transition-all duration-300 ${flowInternal ? 'opacity-100' : 'opacity-30'}`}>
              <span className="size-1.5 rounded-full bg-emerald-400" />
              <span className="text-[9px] text-emerald-400 font-medium">
                {flowInternal ? 'complete' : 'routing'}
              </span>
            </div>
            <FlowArrow active={flowRight} direction="right" />
          </div>
        </div>

        {/* Agent side */}
        <div className={`rounded-xl border bg-card/40 p-4 transition-all duration-500 ${
          current.from === 'agent' || current.to === 'agent'
            ? 'border-sky-500/40 shadow-[0_0_20px_-5px] shadow-sky-500/10'
            : 'border-border/40'
        }`}>
          <div className="text-[10px] font-semibold text-sky-400/70 uppercase tracking-wider mb-3">
            Agents
          </div>
          <div className="space-y-1.5">
            {AGENTS.map((agent, i) => (
              <div
                key={agent.name}
                className={`flex items-center gap-2 rounded-md px-2 py-1.5 text-[11px] transition-all duration-300 ${
                  i === 0 && (current.id === 'assign' || current.id === 'work' || current.id === 'gate')
                    ? 'bg-sky-500/10 text-sky-400 font-medium'
                    : 'text-muted-foreground/50'
                }`}
              >
                <span className={`size-1.5 rounded-full transition-colors duration-300 ${
                  i === 0 && (current.id === 'assign' || current.id === 'work' || current.id === 'gate')
                    ? 'bg-sky-400'
                    : 'bg-muted-foreground/20'
                }`} />
                {agent.label}
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Pipeline step indicators */}
      <div className="mt-6 flex items-center justify-center gap-1">
        {PIPELINE_STAGES.map((s, i) => (
          <div
            key={s.id}
            className={`h-1 rounded-full transition-all duration-300 ${
              i < stage ? 'w-6 bg-emerald-500/60'
                : i === stage ? 'w-8 bg-primary'
                : 'w-4 bg-muted-foreground/15'
            }`}
          />
        ))}
      </div>
    </div>
  )
}

function FlowArrow({ active, direction }: { active: boolean; direction: 'left' | 'right' }) {
  return (
    <div className={`flex items-center gap-0.5 transition-all duration-300 ${active ? 'opacity-100' : 'opacity-20'}`}>
      {direction === 'left' && <span className="text-[10px] text-primary">◄</span>}
      <div className="flex gap-0.5">
        <span className={`size-1 rounded-full ${active ? 'bg-primary animate-pulse' : 'bg-muted-foreground/30'}`} />
        <span className={`size-1 rounded-full ${active ? 'bg-primary/70 animate-pulse' : 'bg-muted-foreground/20'}`} style={{ animationDelay: '150ms' }} />
        <span className={`size-1 rounded-full ${active ? 'bg-primary/40 animate-pulse' : 'bg-muted-foreground/10'}`} style={{ animationDelay: '300ms' }} />
      </div>
      {direction === 'right' && <span className="text-[10px] text-primary">►</span>}
    </div>
  )
}
