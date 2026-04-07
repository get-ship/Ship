import { ArrowRight } from 'lucide-react'

const STEPS = [
  {
    num: '1',
    title: 'Define your agents',
    description:
      'Pick skills, permissions, and MCP servers in Studio. Or install a ready-made config from the registry with ',
    cmd: 'ship add',
    descriptionAfter: '.',
  },
  {
    num: '2',
    title: 'Dispatch work',
    description:
      'Create jobs from the Studio UI or CLI. Ship creates worktrees, assigns pipelines, and launches agents in isolated workspaces.',
  },
  {
    num: '3',
    title: 'Review and ship',
    description:
      'Agents produce artifacts. You review, annotate, and promote. When it looks right, merge. The human stays in control.',
  },
]

export function HowItWorks() {
  return (
    <section className="mx-auto max-w-[62rem] px-6 pb-20 sm:px-10">
      <div className="mb-12 text-center">
        <h2 className="mb-2 font-display text-3xl font-extrabold sm:text-4xl">
          How you work with Ship
        </h2>
        <p className="text-[15px] text-muted-foreground">
          Define. Dispatch. Review. The human-agent workflow.
        </p>
      </div>

      <div className="grid gap-4 sm:grid-cols-3">
        {STEPS.map((step, i) => (
          <div
            key={step.num}
            className="relative rounded-xl border border-border/60 bg-card/30 p-6"
          >
            <div className="mb-3 flex size-7 items-center justify-center rounded-lg bg-primary/10 text-[13px] font-bold text-primary">
              {step.num}
            </div>
            <h3 className="mb-1.5 text-base font-semibold">{step.title}</h3>
            <p className="text-xs leading-relaxed text-muted-foreground">
              {step.description}
              {step.cmd && (
                <code className="rounded bg-muted px-1 py-0.5 text-[11px] text-emerald-400">
                  {step.cmd}
                </code>
              )}
              {step.descriptionAfter}
            </p>
            {/* Arrow connector */}
            {i < STEPS.length - 1 && (
              <div className="pointer-events-none absolute -right-3 top-1/2 hidden -translate-y-1/2 text-border/40 sm:block">
                <ArrowRight className="size-4" />
              </div>
            )}
          </div>
        ))}
      </div>
    </section>
  )
}
