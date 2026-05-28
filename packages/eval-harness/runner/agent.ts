// Agent backend — the SEAM that turns a (prompt, skill, profile) into
// artifacts on disk. Two implementations land here:
//
// 1. `stubAgent` — writes a hand-crafted reference artifact for the
//    case. Used for harness CI: ensures the scorer is wired correctly
//    without paying for real agent calls.
// 2. `claudeCodeAgent` — dispatches via `spawn-agent/dispatch.sh` to a
//    real Claude Code agent in a worktree, captures the files it writes.
//    Slower; the canonical scoring path.
//
// More backends (codex, opencode, gemini) plug in here by implementing
// `AgentBackend`. The eval-matrix mode shells out to each in turn.

import { existsSync, readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { resolve, dirname } from 'node:path'

import type { AgentBackend, AgentRequest, AgentResponse } from './types'

/** Read a hand-crafted reference artifact and copy it into workDir.
 *  Useful to validate the scoring pipeline without dispatching a real
 *  agent. Returns the artifact paths so the scorer can grade them. */
export const stubAgent: AgentBackend = async (req: AgentRequest): Promise<AgentResponse> => {
  // Convention: a stub reference lives at
  // `references/<caseId>/stub-artifact.sui` (and optionally
  // `stub-artifact.controller.ts`). The scorer treats these as a
  // perfect-agent emission for the case.
  const harnessRoot = resolve(dirname(import.meta.url.replace('file://', '')), '..')
  const stubDir = resolve(harnessRoot, 'references', req.caseId)
  const sui = resolve(stubDir, 'stub-artifact.sui')
  if (!existsSync(sui)) {
    throw new Error(`stub agent: no stub-artifact.sui under ${stubDir}; author one to enable stub mode`)
  }
  const artifacts: string[] = []
  for (const name of ['stub-artifact.sui', 'stub-artifact.controller.ts']) {
    const src = resolve(stubDir, name)
    if (!existsSync(src)) continue
    const dst = resolve(req.workDir, name.replace('stub-artifact', req.caseId))
    mkdirSync(dirname(dst), { recursive: true })
    writeFileSync(dst, readFileSync(src, 'utf8'))
    artifacts.push(dst)
  }
  return {
    artifacts,
    telemetry: { backend: 'stub', skill: req.skillVariant, profile: req.agentProfile },
    notes: 'stub backend — copied reference artifact verbatim',
  }
}

/** Dispatch a real Claude Code agent via spawn-agent. The runner
 *  builds a one-shot job spec from the case prompt, dispatches it to a
 *  worktree, waits for completion, then captures the artifacts the
 *  agent wrote. */
export const claudeCodeAgent: AgentBackend = async (_req: AgentRequest): Promise<AgentResponse> => {
  // Implementation lands when the harness graduates from v0. Today the
  // skill loadout per-variant + the agent-profile selection both need
  // the dispatch.sh script extended to accept `--skill-override` and
  // `--profile-override` arguments. Both are tracked in #181's eval-
  // matrix TODO; the seam stays here so the day-of wiring is trivial.
  throw new Error(
    'claudeCodeAgent: not yet wired — extend scripts/dispatch.sh with ' +
      '--skill-override / --profile-override and the runner gains the ' +
      'dispatch step here.',
  )
}

export const backends: Record<string, AgentBackend> = {
  stub: stubAgent,
  claude: claudeCodeAgent,
}
