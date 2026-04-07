import { useState, useEffect } from 'react'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'

const STAGES: { label: string; lit: string[]; agentLit: number[]; duration: number }[] = [
  { label: 'Job dispatched', lit: ['studio'], agentLit: [0], duration: 2500 },
  { label: 'Agents working', lit: ['mesh', 'supervisor'], agentLit: [0, 1], duration: 3000 },
  { label: 'Artifacts staged', lit: ['kernel'], agentLit: [0], duration: 2500 },
  { label: 'Human reviews', lit: ['studio'], agentLit: [], duration: 3000 },
  { label: 'Gate check', lit: ['pipelines'], agentLit: [2], duration: 2500 },
  { label: 'Shipped', lit: ['kernel', 'pipelines'], agentLit: [], duration: 3000 },
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
    const timer = setTimeout(() => {
      setStage((s) => (s + 1) % STAGES.length)
    }, STAGES[stage].duration)
    return () => clearTimeout(timer)
  }, [stage, visible])

  const current = STAGES[stage]

  return (
    <div id="runtime-diagram" className="relative mx-auto max-w-[56rem] px-4">
      {/* Status */}
      <div className="mb-8 flex items-center justify-center gap-2">
        <span className={`size-2 rounded-full ${stage === STAGES.length - 1 ? 'bg-emerald-500' : 'bg-primary'} ${visible ? 'animate-pulse' : ''}`} />
        <span className="text-sm font-medium text-muted-foreground">{current.label}</span>
      </div>

      <div className="flex items-stretch gap-4">
        {/* Studio */}
        <div className={`flex-1 rounded-xl border p-5 transition-all duration-700 ${
          current.lit.includes('studio')
            ? 'border-primary/30 bg-primary/[0.03]'
            : 'border-border/30 bg-card/20'
        }`}>
          <div className="text-[10px] font-semibold text-primary/60 uppercase tracking-wider mb-3">Studio</div>
          <div className="text-xs text-muted-foreground/50 space-y-1.5">
            <div className={current.label === 'Human reviews' ? 'text-primary font-medium' : ''}>Review</div>
            <div className={current.label === 'Job dispatched' ? 'text-primary font-medium' : ''}>Dispatch</div>
            <div>Annotate</div>
            <div>Chat</div>
          </div>
        </div>

        {/* Runtime */}
        <div className="flex-[2] rounded-xl border border-border/40 bg-card/30 p-5">
          <div className="text-[10px] font-semibold text-muted-foreground/50 uppercase tracking-wider mb-3 text-center">
            Ship Runtime
          </div>
          <div className="grid grid-cols-2 gap-2">
            {(['kernel', 'supervisor', 'mesh', 'pipelines'] as const).map((svc) => (
              <div key={svc} className={`rounded-lg border px-3 py-2 text-center transition-all duration-500 ${
                current.lit.includes(svc) || svc === 'kernel'
                  ? 'border-border/50 bg-background/60'
                  : 'border-border/20 bg-background/20'
              }`}>
                <span className={`text-[10px] font-medium capitalize transition-opacity duration-500 ${
                  current.lit.includes(svc) || svc === 'kernel' ? 'opacity-100' : 'opacity-30'
                } ${
                  svc === 'kernel' ? 'text-primary' : svc === 'supervisor' ? 'text-sky-400' : svc === 'mesh' ? 'text-emerald-400' : 'text-amber-400'
                }`}>
                  {svc === 'kernel' ? 'Event Kernel' : svc === 'supervisor' ? 'Supervisor' : svc === 'mesh' ? 'Service Mesh' : 'Pipelines'}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* Agents — just three boxes with provider icons */}
        <div className="flex-1 flex flex-col gap-2">
          {[
            { id: 'claude', name: 'Claude Code' },
            { id: 'gemini', name: 'Gemini CLI' },
            { id: 'codex', name: 'Codex CLI' },
          ].map((agent, i) => (
            <div
              key={agent.id}
              className={`flex-1 rounded-xl border flex items-center justify-center gap-2.5 px-4 transition-all duration-500 ${
                current.agentLit.includes(i)
                  ? 'border-sky-500/30 bg-sky-500/[0.04] shadow-[0_0_20px_-8px] shadow-sky-500/15'
                  : 'border-border/25 bg-card/15'
              }`}
            >
              <div className={`transition-opacity duration-500 ${current.agentLit.includes(i) ? 'opacity-100' : 'opacity-25'}`}>
                <ProviderLogo provider={agent.id} size="md" />
              </div>
              <span className={`text-[11px] font-medium transition-all duration-500 ${
                current.agentLit.includes(i) ? 'text-foreground/80' : 'text-muted-foreground/25'
              }`}>
                {agent.name}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* Progress dots */}
      <div className="mt-8 flex items-center justify-center gap-1.5">
        {STAGES.map((_, i) => (
          <div key={i} className={`h-1 rounded-full transition-all duration-500 ${
            i < stage ? 'w-6 bg-emerald-500/50' : i === stage ? 'w-10 bg-primary' : 'w-4 bg-muted-foreground/10'
          }`} />
        ))}
      </div>
    </div>
  )
}
