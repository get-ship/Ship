import { ProviderLogo } from '#/features/compiler/ProviderLogo'

// ── Section wrapper — alternates text/visual sides ──

function Section({ children, id }: { children: React.ReactNode; id?: string }) {
  return (
    <section id={id} className="px-6 py-14 max-w-5xl mx-auto sm:px-10">
      {children}
    </section>
  )
}

function SplitRow({ visual, children, reverse }: { visual: React.ReactNode; children: React.ReactNode; reverse?: boolean }) {
  return (
    <div className={`flex flex-col gap-8 items-center sm:flex-row sm:gap-12 ${reverse ? 'sm:flex-row-reverse' : ''}`}>
      <div className="flex-1 min-w-0">{children}</div>
      <div className="flex-1 min-w-0 flex justify-center">{visual}</div>
    </div>
  )
}

function Tag({ children }: { children: React.ReactNode }) {
  return <span className="text-[10px] font-semibold text-primary/70 uppercase tracking-wider">{children}</span>
}

function Title({ children }: { children: React.ReactNode }) {
  return <h3 className="font-display text-2xl font-extrabold mt-1.5 mb-3 sm:text-3xl">{children}</h3>
}

function Desc({ children }: { children: React.ReactNode }) {
  return <p className="text-[15px] leading-relaxed text-muted-foreground max-w-md">{children}</p>
}

// ── Visual components ──

function CompilerVisual() {
  const providers = [
    { id: 'claude', file: 'CLAUDE.md' },
    { id: 'cursor', file: '.cursor/' },
    { id: 'gemini', file: 'GEMINI.md' },
    { id: 'codex', file: '.codex/' },
    { id: 'opencode', file: 'opencode.json' },
  ]
  return (
    <div className="w-full max-w-xs">
      {/* Input */}
      <div className="rounded-lg border border-primary/20 bg-primary/5 px-4 py-2.5 mb-3">
        <div className="text-[10px] text-primary/60 font-mono mb-1">.ship/</div>
        <div className="text-xs font-medium text-primary font-mono">agents/ skills/ rules/</div>
      </div>
      {/* Arrow */}
      <div className="flex justify-center mb-3">
        <div className="text-muted-foreground/30 text-lg">↓</div>
      </div>
      {/* Outputs */}
      <div className="space-y-1.5">
        {providers.map((p) => (
          <div key={p.id} className="flex items-center gap-2.5 rounded-lg border border-border/40 bg-card/50 px-3 py-2">
            <ProviderLogo provider={p.id} size="sm" />
            <span className="text-xs font-mono text-muted-foreground">{p.file}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

function RegistryVisual() {
  return (
    <div className="w-full max-w-xs rounded-lg border border-border/40 bg-card/50 overflow-hidden">
      <div className="px-3 py-2 border-b border-border/30 text-[10px] font-mono text-muted-foreground/50">
        $ ship add
      </div>
      <div className="p-3 space-y-2">
        {['code-review', 'tdd', 'better-auth'].map((skill) => (
          <div key={skill} className="flex items-center gap-2 text-xs">
            <span className="size-1.5 rounded-full bg-emerald-500/60" />
            <span className="font-mono text-foreground/80">{skill}</span>
            <span className="ml-auto text-[9px] text-muted-foreground/40">installed</span>
          </div>
        ))}
        {['deploy-tools', 'monitoring'].map((skill) => (
          <div key={skill} className="flex items-center gap-2 text-xs">
            <span className="size-1.5 rounded-full bg-muted-foreground/20" />
            <span className="font-mono text-muted-foreground/50">{skill}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

function ArtifactsVisual() {
  return (
    <div className="w-full max-w-xs">
      <div className="rounded-lg border border-border/40 bg-card/50 p-3 relative">
        <div className="text-[10px] font-medium mb-2 text-foreground/70">design-spec.html</div>
        {/* Mini rendered HTML with real-looking content */}
        <div className="rounded bg-background/60 border border-border/20 p-3 relative" style={{ minHeight: 100 }}>
          <div className="text-[9px] font-display font-bold text-foreground/80 mb-1">Navigation Redesign</div>
          <div className="text-[8px] text-muted-foreground/50 mb-2">Rail-based layout with workspace scoping</div>
          <div className="flex gap-1.5 mb-2">
            <div className="h-5 w-8 rounded-sm bg-primary/15 border border-primary/20" />
            <div className="flex-1 space-y-1">
              <div className="h-2 w-3/4 rounded-full bg-muted/50" />
              <div className="h-2 w-1/2 rounded-full bg-muted/30" />
            </div>
          </div>
          <div className="h-6 rounded bg-muted/20 border border-border/10" />
          {/* Annotation marker */}
          <div className="absolute top-2 right-2 size-5 rounded-full bg-primary flex items-center justify-center text-[8px] font-bold text-primary-foreground shadow-[0_0_0_2px] shadow-primary/25">
            1
          </div>
          {/* Annotation note */}
          <div className="absolute top-8 right-2 bg-primary/10 border border-primary/20 rounded px-1.5 py-0.5 text-[7px] text-primary max-w-[80px]">
            "Move this to the rail"
          </div>
        </div>
        <div className="mt-2 flex items-center justify-center gap-1">
          <span className="px-2 py-0.5 rounded-full text-[8px] text-muted-foreground/50 border border-border/30">Annotate</span>
          <span className="px-2 py-0.5 rounded-full text-[8px] font-medium text-primary bg-primary/10 border border-primary/20">Send to Agent</span>
        </div>
      </div>
      <div className="flex items-center gap-2 mt-2 px-1">
        <span className="size-1.5 rounded-full bg-emerald-500/60" />
        <span className="text-[9px] text-muted-foreground/50">Promoted to project</span>
      </div>
    </div>
  )
}

function PipelinesVisual() {
  const steps = [
    { id: 'test', status: 'done' },
    { id: 'impl', status: 'done' },
    { id: 'gate', status: 'active' },
    { id: 'merge', status: 'pending' },
  ]
  return (
    <div className="w-full max-w-xs rounded-lg border border-border/40 bg-card/50 p-4">
      <div className="flex items-center gap-2 mb-3">
        <span className="text-[10px] font-mono text-foreground/70">tdd.yaml</span>
        <span className="ml-auto text-[9px] text-sky-400">running</span>
      </div>
      <div className="flex items-center gap-2">
        {steps.map((step, i) => (
          <div key={step.id} className="flex items-center gap-2">
            <div className={`px-2.5 py-1 rounded text-[10px] font-medium ${
              step.status === 'done' ? 'bg-emerald-500/10 text-emerald-500'
              : step.status === 'active' ? 'bg-sky-500/10 text-sky-400 ring-1 ring-sky-500/30'
              : 'bg-muted/30 text-muted-foreground/40'
            }`}>
              {step.id}
            </div>
            {i < steps.length - 1 && (
              <span className="text-muted-foreground/20 text-[10px]">→</span>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}

function SkillsVisual() {
  return (
    <div className="w-full max-w-xs rounded-lg border border-border/40 bg-card/50 overflow-hidden">
      <div className="px-3 py-1.5 border-b border-border/30 text-[10px] font-mono text-muted-foreground/50">
        SKILL.md
      </div>
      <div className="p-3 font-mono text-[10px] leading-relaxed text-muted-foreground/70">
        <div className="text-primary/50">---</div>
        <div><span className="text-primary/70">name:</span> code-review</div>
        <div><span className="text-primary/70">description:</span> PR review</div>
        <div><span className="text-primary/70">tags:</span> [quality, gate]</div>
        <div className="text-primary/50">---</div>
        <div className="mt-1 text-foreground/50"># Code Review</div>
        <div className="text-muted-foreground/40">When reviewing a diff...</div>
      </div>
    </div>
  )
}

function PermissionsVisual() {
  const rows = [
    { name: 'Read', state: 'allow' },
    { name: 'Write', state: 'allow' },
    { name: 'Bash(git *)', state: 'ask' },
    { name: 'Bash(rm *)', state: 'deny' },
    { name: 'Bash(deploy *)', state: 'deny' },
  ]
  return (
    <div className="w-full max-w-xs rounded-lg border border-border/40 bg-card/50 p-3">
      <div className="text-[9px] font-semibold text-muted-foreground/40 uppercase tracking-wide mb-2">
        github MCP — web-lane
      </div>
      {rows.map((row) => (
        <div key={row.name} className="flex items-center gap-2 py-1.5 text-[11px]">
          <div className={`relative h-3 w-5 shrink-0 rounded-full ${
            row.state === 'allow' ? 'bg-emerald-400' : row.state === 'ask' ? 'bg-primary' : 'bg-muted-foreground/30'
          }`}>
            <div className={`absolute top-[2px] size-2 rounded-full bg-white ${
              row.state === 'deny' ? 'left-[2px]' : 'left-[10px]'
            }`} />
          </div>
          <span className={`flex-1 font-mono text-[10px] ${row.state === 'deny' ? 'text-muted-foreground/25 line-through' : 'text-muted-foreground/70'}`}>
            {row.name}
          </span>
          <span className={`text-[8px] font-medium px-1.5 py-0.5 rounded ${
            row.state === 'allow' ? 'bg-emerald-500/10 text-emerald-400'
            : row.state === 'ask' ? 'bg-primary/10 text-primary'
            : 'bg-red-500/10 text-red-400'
          }`}>
            {row.state}
          </span>
        </div>
      ))}
    </div>
  )
}

// ── Export ──

export function FeatureSections() {
  return (
    <div className="border-t border-border/30">
      <div className="text-center pt-16 pb-4 px-6">
        <h2 className="font-display text-3xl font-extrabold sm:text-4xl mb-2">
          A complete platform for working with agents
        </h2>
        <p className="text-[15px] text-muted-foreground max-w-lg mx-auto">
          Compiler, runtime, and GUI working together. Open in your browser, served from localhost.
        </p>
      </div>

      <Section>
        <SplitRow visual={<CompilerVisual />}>
          <Tag>Compiler</Tag>
          <Title>Build a roster of specialists</Title>
          <Desc>
            Each agent gets its own tools, permissions, and instructions. Ship compiles the whole roster to native config for every provider you use.
          </Desc>
        </SplitRow>
      </Section>

      <Section>
        <SplitRow visual={<RegistryVisual />} reverse>
          <Tag>Registry</Tag>
          <Title>Install with one command</Title>
          <Desc>
            Skills, MCP servers, and agent configs — versioned, audited, and ready to use.
          </Desc>
        </SplitRow>
      </Section>

      <Section>
        <SplitRow visual={<ArtifactsVisual />}>
          <Tag>Artifacts & Annotations</Tag>
          <Title>Review before it ships</Title>
          <Desc>
            Agents produce work into a staging area. Draw on canvases, comment on diffs, promote to the project when it's ready. Nothing lands without your sign-off.
          </Desc>
        </SplitRow>
      </Section>

      <Section>
        <SplitRow visual={<PipelinesVisual />} reverse>
          <Tag>Jobs & Pipelines</Tag>
          <Title>Structured dispatch</Title>
          <Desc>
            Define workflows as YAML DAGs. Each job gets its own worktree and agent process. Track progress in real time.
          </Desc>
        </SplitRow>
      </Section>

      <Section>
        <SplitRow visual={<SkillsVisual />}>
          <Tag>Skills</Tag>
          <Title>The open standard for agent instructions</Title>
          <Desc>
            Ship extends Agent Skills with template variables, artifacts, and a full IDE. Install from the registry or write your own.
          </Desc>
        </SplitRow>
      </Section>

      <Section>
        <SplitRow visual={<PermissionsVisual />} reverse>
          <Tag>Permissions</Tag>
          <Title>Every tool, individually scoped</Title>
          <Desc>
            Control exactly which tools each agent can use — allow, ask, or deny at the individual tool level across every MCP server.
          </Desc>
        </SplitRow>
      </Section>
    </div>
  )
}
