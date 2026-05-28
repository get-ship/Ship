// Shared type vocabulary for the eval harness. Kept tiny on purpose —
// every type here crosses corpus + runner + scorer + result, so a
// breaking change costs three places.

export type CaseKind = 'view' | 'component'

export interface CaseMeta {
  id: string
  kind: CaseKind
  /** Custom-element tag (component cases only). */
  tag?: string
  /** ID under `references/`; usually equals `id`. */
  reference: string
  /** Score weights — must sum to 1.0 (validated). */
  weights: {
    compile: number
    lint: number
    structural: number
    render: number
  }
  notes?: string
}

export interface CaseFixture {
  meta: CaseMeta
  /** Absolute path to the case's `prompt.md`. */
  promptPath: string
  /** Absolute path to the case's reference dir. */
  referencePath: string
}

export interface AgentRequest {
  /** The case under evaluation. */
  caseId: string
  /** Markdown contents of the case's `prompt.md`. */
  prompt: string
  /** Which SKILL.md variant to load. The agent backend reads this. */
  skillVariant: string
  /** Identifier for the agent + model + system prompt config. */
  agentProfile: string
  /** Absolute path the agent should write artifacts into. */
  workDir: string
}

export interface AgentResponse {
  /** Absolute paths to every file the agent wrote. */
  artifacts: string[]
  /** Token / latency telemetry, opaque to the scorer. */
  telemetry: Record<string, unknown>
  /** Free-form notes the agent surfaced — for human review only. */
  notes?: string
}

export type AgentBackend = (req: AgentRequest) => Promise<AgentResponse>

export interface ScoreCard {
  caseId: string
  /** Per-axis raw results. */
  compile: { passed: boolean; stdout: string; stderr: string }
  lint: { findings: LintFinding[]; errors: number; warnings: number }
  structural: { matched: number; missing: string[]; forbidden: string[] }
  render?: { passed: boolean; details: string }
  /** Composite weighted score in [0, 1]. */
  composite: number
}

export interface LintFinding {
  file: string
  line: number
  col: number
  rule: string
  severity: 'Error' | 'Warn'
  message: string
}

export interface ExpectedReference {
  kind: CaseKind
  tag?: string
  props?: Array<Record<string, unknown>>
  events?: Array<Record<string, unknown>>
  slots?: Array<Record<string, unknown>>
  stateKeys?: string[]
  /** Substrings that MUST NOT appear in any controller file. */
  forbiddenInController?: string[]
  /** Substrings that MUST appear in the template (sui file). */
  requiredInTemplate?: string[]
  notes?: string
}
