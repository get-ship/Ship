// Score an agent's artifact against the case's expected reference.
// Three axes graded; render axis defers to #164. Composite uses the
// case's weight bag (validated to sum to 1.0 at load time).

import { execSync } from 'node:child_process'
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs'
import { resolve, relative } from 'node:path'

import type {
  AgentResponse,
  CaseFixture,
  ExpectedReference,
  LintFinding,
  ScoreCard,
} from './types'

export function scoreArtifact(
  fixture: CaseFixture,
  response: AgentResponse,
  workspaceRoot: string,
): ScoreCard {
  const expected = loadExpected(fixture)
  const compile = scoreCompile(response, workspaceRoot, fixture)
  const lint = scoreLint(compile)
  const structural = scoreStructural(response, expected)

  const w = fixture.meta.weights
  // Render axis is deferred until #164 lands; for now bias compile + lint
  // + structural to fill its weight so the composite isn't artificially low.
  const renderShim = compile.passed && lint.errors === 0 && structural.missing.length === 0 ? 1 : 0
  const composite =
    w.compile    * (compile.passed ? 1 : 0) +
    w.lint       * (lint.errors === 0 ? 1 - clamp01(lint.warnings * 0.05) : 0) +
    w.structural * structuralFraction(structural, expected) +
    w.render     * renderShim

  return {
    caseId: fixture.meta.id,
    compile,
    lint,
    structural,
    composite,
  }
}

function loadExpected(fixture: CaseFixture): ExpectedReference {
  const expectedPath = resolve(fixture.referencePath, 'expected.json')
  if (!existsSync(expectedPath)) {
    throw new Error(`eval-harness: missing expected.json under ${fixture.referencePath}`)
  }
  return JSON.parse(readFileSync(expectedPath, 'utf8')) as ExpectedReference
}

function scoreCompile(
  response: AgentResponse,
  workspaceRoot: string,
  fixture: CaseFixture,
): ScoreCard['compile'] {
  const suiArtifacts = response.artifacts.filter((p) => p.endsWith('.sui'))
  if (suiArtifacts.length === 0) {
    return { passed: false, stdout: '', stderr: 'no .sui artifact emitted' }
  }
  // Compile each .sui via `sui-compiler build`. Pass --component-name when
  // the case is a component (matches the path-convention guardrail #176).
  let stdout = ''
  let stderr = ''
  for (const sui of suiArtifacts) {
    const rel = relative(workspaceRoot, sui)
    const args = ['run', '-p', 'sui-compiler', '--', 'build', rel]
    if (fixture.meta.kind === 'component' && fixture.meta.tag) {
      args.push('--component-name', fixture.meta.tag)
    }
    try {
      stdout += execSync(`cargo ${args.join(' ')}`, {
        cwd: workspaceRoot,
        encoding: 'utf8',
        stdio: ['ignore', 'pipe', 'pipe'],
      })
    } catch (e) {
      const err = e as { stderr?: Buffer | string; stdout?: Buffer | string }
      stderr += (err.stderr?.toString() ?? '') + '\n'
      stdout += (err.stdout?.toString() ?? '')
      return { passed: false, stdout, stderr }
    }
  }
  return { passed: true, stdout, stderr }
}

function scoreLint(compile: ScoreCard['compile']): ScoreCard['lint'] {
  // sui-lint findings flow through the compiler's diagnostic stream;
  // stderr/stdout carry them when compile failed. For the v0 scorer we
  // treat compile-stderr as the lint signal; a richer parser lands when
  // we wire sui-lint as a standalone bin.
  const findings: LintFinding[] = []
  let errors = 0
  let warnings = 0
  const text = compile.stdout + compile.stderr
  // Match `path:line:col: <severity>: <message> [rule]`
  const re = /^(.+?):(\d+):(\d+):\s+(error|warning|warn):\s+(.+?)(?:\s+\[([\w-]+)\])?$/gm
  let m: RegExpExecArray | null
  while ((m = re.exec(text)) !== null) {
    const sev = m[4]!.toLowerCase() === 'error' ? 'Error' : 'Warn'
    findings.push({
      file: m[1]!,
      line: Number(m[2]!),
      col: Number(m[3]!),
      severity: sev,
      message: m[5]!,
      rule: m[6] ?? '',
    })
    if (sev === 'Error') errors++
    else warnings++
  }
  return { findings, errors, warnings }
}

function scoreStructural(
  response: AgentResponse,
  expected: ExpectedReference,
): ScoreCard['structural'] {
  // Read every text artifact stripped of comments. Comments often
  // document what NOT to do (e.g. "never use state.x.set(v) here"),
  // which would false-positive a forbidden-substring check. The agent's
  // CODE is the signal; their comments are advisory.
  const sources = response.artifacts.filter(
    (p) => p.endsWith('.sui') || p.endsWith('.ts') || p.endsWith('.css'),
  )
  const allText = sources.map((p) => stripComments(p, safeRead(p))).join('\n──\n')

  const required = expected.requiredInTemplate ?? []
  const forbidden = expected.forbiddenInController ?? []
  const missing = required.filter((s) => !allText.includes(s))
  const present = required.length - missing.length
  const forbiddenHits = forbidden.filter((s) => allText.includes(s))
  return { matched: present, missing, forbidden: forbiddenHits }
}

/** Strip comments by file extension. Conservative: removes line `//` and
 *  block `/* * /` from .ts/.css and `<!-- -->` from .sui (which is HTML-
 *  shaped). Preserves string contents (a naive `//` strip would corrupt
 *  URLs in string literals). */
function stripComments(path: string, src: string): string {
  if (path.endsWith('.sui')) return stripHtmlComments(src)
  return stripJsLikeComments(src)
}

function stripHtmlComments(src: string): string {
  return src.replace(/<!--[\s\S]*?-->/g, '')
}

function stripJsLikeComments(src: string): string {
  // State machine: track string/template-literal contexts so `//` inside a
  // string isn't treated as a comment.
  let out = ''
  let i = 0
  const n = src.length
  let inStr: string | null = null
  while (i < n) {
    const ch = src[i]!
    const nx = src[i + 1]
    if (inStr) {
      out += ch
      if (ch === '\\') { out += nx ?? ''; i += 2; continue }
      if (ch === inStr) inStr = null
      i++
      continue
    }
    if (ch === '"' || ch === '\'' || ch === '`') {
      inStr = ch
      out += ch
      i++
      continue
    }
    if (ch === '/' && nx === '/') {
      // Line comment — skip to next newline (keep the newline so line
      // numbers in error messages stay accurate).
      const eol = src.indexOf('\n', i)
      i = eol === -1 ? n : eol
      continue
    }
    if (ch === '/' && nx === '*') {
      const end = src.indexOf('*/', i + 2)
      i = end === -1 ? n : end + 2
      continue
    }
    out += ch
    i++
  }
  return out
}

function structuralFraction(
  s: ScoreCard['structural'],
  expected: ExpectedReference,
): number {
  const required = expected.requiredInTemplate ?? []
  if (required.length === 0) return s.forbidden.length === 0 ? 1 : 0
  const presence = (required.length - s.missing.length) / required.length
  // Forbidden hits zero the structural score outright — they're the
  // anti-patterns we built the harness to detect.
  return s.forbidden.length === 0 ? presence : 0
}

function safeRead(path: string): string {
  try {
    return readFileSync(path, 'utf8')
  } catch {
    return ''
  }
}

function clamp01(n: number): number {
  return Math.max(0, Math.min(1, n))
}

// Re-export for the CLI's quick rollup output.
export function collectArtifactPaths(workDir: string): string[] {
  const out: string[] = []
  function walk(dir: string): void {
    if (!existsSync(dir)) return
    for (const entry of readdirSync(dir)) {
      const p = resolve(dir, entry)
      if (statSync(p).isDirectory()) walk(p)
      else out.push(p)
    }
  }
  walk(workDir)
  return out
}
