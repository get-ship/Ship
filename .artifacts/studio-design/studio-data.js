/* ===== Studio data =====
 * Realistic-shaped data per Ship schema. Keep flat arrays so render is trivial.
 */

// Rail definition — grouped, with glyph hues
const DOMAINS = [
  { group: 'Identities',  items: [
    { id: 'agents',      label: 'Agents',      glyph: 'A', hue: 'orange', count: 8 },
  ]},
  { group: 'Capabilities', items: [
    { id: 'skills',      label: 'Skills',      glyph: 'S', hue: 'violet', count: 14 },
    { id: 'mcp',         label: 'MCP servers', glyph: 'M', hue: 'blue',   count: 5 },
  ]},
  { group: 'Workflows', items: [
    { id: 'pipelines',   label: 'Pipelines',   glyph: 'P', hue: 'teal',   count: 6 },
  ]},
  { group: 'UI', items: [
    { id: 'components',  label: 'Components',  glyph: 'C', hue: 'green',  count: 80 },
    { id: 'themes',      label: 'Themes',      glyph: 'T', hue: 'pink',   count: 7 },
  ]},
  { group: 'Policy', items: [
    { id: 'permissions', label: 'Permissions', glyph: '!', hue: 'red',    count: 4 },
  ]},
];

// ───────────────── AGENTS ─────────────────
const AGENTS = [
  { id: 'rust-compiler', name: 'Rust Compiler', hue: 'orange', letter: 'R',
    description: 'Compiles Rust workspaces, reports clippy spans inline, blocks PRs on warnings.',
    skills: ['tdd', 'lint', 'build'], mcp: ['filesystem', 'github'],
    rules_count: 5, providers: ['anthropic', 'openai'], tags: ['workflow', 'rust', 'build', 'review'],
    source: 'project', version: '0.4.2', role: 'worker',
    preset: 'project-write',
  },
  { id: 'doc-reviewer', name: 'Doc Reviewer', hue: 'blue', letter: 'D',
    description: 'Reads PRs and proposes edits for clarity, structure, and missing context.',
    skills: ['code-review', 'doc-pdf', 'format-md', 'regex-craft', 'accessibility-audit'],
    mcp: ['github'], rules_count: 3, providers: ['anthropic'], tags: ['review', 'docs'],
    source: 'project', version: '0.2.1', role: 'lead',
    preset: 'project-read',
  },
  { id: 'test-runner', name: 'Test Runner', hue: 'green', letter: 'T',
    description: 'Runs unit + integration suites with auto-retry on flaky failures.',
    skills: ['tdd', 'perf-trace'], mcp: [], rules_count: 4, providers: ['anthropic', 'gemini'],
    tags: ['testing'], source: 'project', version: '1.0.0', role: 'worker',
    preset: 'project-write',
  },
  { id: 'etl-runner', name: 'ETL Runner', hue: 'violet', letter: 'E',
    description: 'Daily extract + load on warehouse partitions; emits collection.* events.',
    skills: ['sql-optimize', 'db-migrate', 'regex-craft', 'perf-trace'],
    mcp: ['postgres', 'filesystem', 'brave-search'], rules_count: 2, providers: ['anthropic'],
    tags: ['data', 'etl'], source: 'project', version: '0.6.0', role: 'worker',
    preset: 'project-write',
  },
  { id: 'mesh-sentinel', name: 'Mesh Sentinel', hue: 'teal', letter: 'M',
    description: 'Watches mesh topology and alerts on partition or degraded nodes.',
    skills: ['threat-model'], mcp: ['filesystem', 'brave-search'],
    rules_count: 1, providers: ['openai'], tags: ['ops', 'mesh'],
    source: 'user', version: '0.3.0', role: 'worker',
    preset: 'ops',
  },
  { id: 'sql-shaper', name: 'SQL Shaper', hue: 'amber', letter: 'S',
    description: 'Rewrites query plans; suggests indexes from EXPLAIN output.',
    skills: ['sql-optimize', 'perf-trace'], mcp: ['postgres'], rules_count: 3,
    providers: ['anthropic'], tags: ['data', 'sql'],
    source: 'project', version: '0.5.0', role: 'worker',
    preset: 'project-write',
  },
  { id: 'prompt-tuner', name: 'Prompt Tuner', hue: 'pink', letter: 'P',
    description: 'Eval harness for prompt regressions across model providers.',
    skills: ['html-prototype', 'api-design', 'threat-model'], mcp: [], rules_count: 2,
    providers: ['anthropic', 'openai', 'gemini'], tags: ['eval'],
    source: 'user', version: '0.1.0', role: 'lead',
    preset: 'project-read',
  },
  { id: 'code-formatter', name: 'Code Formatter', hue: 'slate', letter: 'F',
    description: 'Applies project-wide style on pre-commit; respects per-lang configs.',
    skills: ['format-md'], mcp: [], rules_count: 1, providers: ['anthropic'],
    tags: ['utility', 'lint'], source: 'project', version: '1.2.0', role: 'worker',
    preset: 'project-write',
  },
];

// ───────────────── SKILLS ─────────────────
const SKILLS = [
  { id: 'tdd', name: 'tdd', hue: 'violet', letter: 'T', version: '1.2',
    description: 'Use when implementing a new behaviour — write failing test first, then iterate to green.',
    tags: ['testing', 'workflow', 'core'], authors: ['platform'], audience: 'public',
    artifacts: ['markdown'], compat: 'universal', used_by: 2 },
  { id: 'lint', name: 'lint', hue: 'amber', letter: 'L', version: '0.8',
    description: 'Apply clippy lints with project-defined severity overrides; emit fixable diffs.',
    tags: ['rust', 'review'], authors: ['platform'], audience: 'public',
    artifacts: ['json', 'markdown'], compat: 'anthropic', used_by: 1 },
  { id: 'build', name: 'build', hue: 'orange', letter: 'B', version: '2.0',
    description: 'Use when compiling Rust workspaces — cache-aware artifact upload to run scope.',
    tags: ['rust', 'build'], authors: ['platform'], audience: 'public',
    artifacts: ['json'], compat: 'universal', used_by: 1 },
  { id: 'code-review', name: 'code-review', hue: 'blue', letter: 'C', version: '1.4',
    description: 'Use when reading diffs — comments on intent drift, edge cases, missing tests.',
    tags: ['review', 'code'], authors: ['platform'], audience: 'public',
    artifacts: ['markdown'], compat: 'universal', used_by: 3 },
  { id: 'regex-craft', name: 'regex-craft', hue: 'pink', letter: 'R', version: '0.5',
    description: 'Use when matching/extracting text — produces minimal-spec regex with tests.',
    tags: ['text', 'utility'], authors: ['malik'], audience: 'public',
    artifacts: ['markdown', 'json'], compat: 'universal', used_by: 2 },
  { id: 'sql-optimize', name: 'sql-optimize', hue: 'amber', letter: 'Q', version: '1.0',
    description: 'Use when an EXPLAIN looks expensive — suggests rewrites and indexes.',
    tags: ['data', 'sql'], authors: ['platform'], audience: 'public',
    artifacts: ['markdown', 'json'], compat: 'anthropic, openai', used_by: 2 },
  { id: 'doc-pdf', name: 'doc-pdf', hue: 'slate', letter: 'D', version: '0.3',
    description: 'Use when delivering a polished read-only document — converts markdown → PDF.',
    tags: ['docs', 'export'], authors: ['malik'], audience: 'public',
    artifacts: ['pdf', 'markdown'], compat: 'universal', used_by: 1 },
  { id: 'format-md', name: 'format-md', hue: 'green', letter: 'F', version: '1.1',
    description: 'Normalize markdown — heading levels, table alignment, code-block fences.',
    tags: ['docs', 'utility'], authors: ['platform'], audience: 'public',
    artifacts: ['markdown'], compat: 'universal', used_by: 2 },
  { id: 'html-prototype', name: 'html-prototype', hue: 'orange', letter: 'H', version: '0.7',
    description: 'Use when sketching a UI — produces a single-file static HTML mock.',
    tags: ['ui', 'prototyping'], authors: ['malik'], audience: 'public',
    artifacts: ['html'], compat: 'universal', used_by: 1 },
  { id: 'api-design', name: 'api-design', hue: 'teal', letter: 'A', version: '0.4',
    description: 'Use when proposing endpoints — output an OpenAPI fragment + rationale.',
    tags: ['backend', 'design'], authors: ['platform'], audience: 'internal',
    artifacts: ['json', 'markdown'], compat: 'anthropic, openai', used_by: 1 },
  { id: 'threat-model', name: 'threat-model', hue: 'red', letter: 'X', version: '0.6',
    description: 'Use when shipping a network-exposed feature — produces a STRIDE table.',
    tags: ['security', 'design'], authors: ['platform'], audience: 'internal',
    artifacts: ['markdown'], compat: 'universal', used_by: 2 },
  { id: 'perf-trace', name: 'perf-trace', hue: 'violet', letter: 'P', version: '0.2',
    description: 'Use when a benchmark regresses — annotate the slow span.',
    tags: ['perf', 'debug'], authors: ['malik'], audience: 'public',
    artifacts: ['json', 'svg'], compat: 'anthropic', used_by: 2 },
  { id: 'accessibility-audit', name: 'accessibility-audit', hue: 'blue', letter: 'a', version: '0.9',
    description: 'Use when reviewing a page — finds aria, contrast, and keyboard-trap issues.',
    tags: ['ui', 'a11y'], authors: ['platform'], audience: 'public',
    artifacts: ['markdown'], compat: 'universal', used_by: 1 },
  { id: 'db-migrate', name: 'db-migrate', hue: 'amber', letter: 'm', version: '1.3',
    description: 'Use when adding a column — emits an idempotent SQL migration.',
    tags: ['data', 'sql'], authors: ['platform'], audience: 'public',
    artifacts: ['url', 'markdown'], compat: 'universal', used_by: 1 },
];

// ───────────────── MCP ─────────────────
const MCPS = [
  { id: 'filesystem', name: 'filesystem', hue: 'blue', letter: 'F',
    transport: 'stdio', scope: 'project', enabled: true,
    description: 'Sandboxed FS read/write for the active workspace root.',
    command: 'npx -y @ship/mcp-filesystem',
    env_count: 1, tool_count: 7, used_by: 4 },
  { id: 'github', name: 'github', hue: 'slate', letter: 'G',
    transport: 'http', scope: 'global', enabled: true,
    description: 'Issues, PRs, repo metadata via GitHub REST + GraphQL.',
    url: 'https://mcp.github.com/v1', env_count: 1, tool_count: 22, used_by: 3 },
  { id: 'postgres', name: 'postgres', hue: 'violet', letter: 'P',
    transport: 'stdio', scope: 'project', enabled: true,
    description: 'Read-only SQL on the warehouse replica; rejects DDL.',
    command: 'mcp-postgres --readonly', env_count: 3, tool_count: 4, used_by: 2 },
  { id: 'brave-search', name: 'brave-search', hue: 'orange', letter: 'B',
    transport: 'http', scope: 'global', enabled: true,
    description: 'Web search; cached results, rate-limited per workspace.',
    url: 'https://api.search.brave.com/mcp', env_count: 1, tool_count: 1, used_by: 2 },
  { id: 'ship-docs', name: 'ship-docs', hue: 'green', letter: 'S',
    transport: 'sse', scope: 'global', enabled: false,
    description: 'Vector search over the canonical Ship docs corpus.',
    url: 'https://docs.getship.dev/mcp', env_count: 0, tool_count: 3, used_by: 0 },
];

// ───────────────── PIPELINES ─────────────────
const PIPELINES = [
  { id: 'release', name: 'release', hue: 'teal', letter: 'R',
    description: 'Build → test → review → tag → publish. Blocks on diff-gate; auto-merges on green.',
    auto_merge: true, vars_count: 4,
    steps: [
      { id: 'build', type: 'agent', agent: 'rust-compiler', after: [] },
      { id: 'test', type: 'agent', agent: 'test-runner', after: ['build'] },
      { id: 'review', type: 'review', agent: 'doc-reviewer', after: ['test'] },
      { id: 'tag', type: 'run', after: ['review'] },
      { id: 'publish', type: 'run', after: ['tag'] },
    ]
  },
  { id: 'weekly-review', name: 'weekly-review', hue: 'blue', letter: 'W',
    description: 'Friday afternoon: doc-review on the past 7 days of merged PRs.',
    auto_merge: false, vars_count: 1,
    steps: [
      { id: 'gather', type: 'run', after: [] },
      { id: 'review', type: 'agent', agent: 'doc-reviewer', after: ['gather'] },
      { id: 'summarize', type: 'agent', agent: 'doc-reviewer', after: ['review'] },
    ]
  },
  { id: 'etl-daily', name: 'etl-daily', hue: 'violet', letter: 'E',
    description: 'Pull → shape → load. Targets warehouse partitions for the previous UTC day.',
    auto_merge: false, vars_count: 6,
    steps: [
      { id: 'extract', type: 'run', after: [] },
      { id: 'shape', type: 'agent', agent: 'sql-shaper', after: ['extract'] },
      { id: 'load', type: 'agent', agent: 'etl-runner', after: ['shape'] },
      { id: 'validate', type: 'agent', agent: 'etl-runner', after: ['load'] },
    ]
  },
  { id: 'doc-publish', name: 'doc-publish', hue: 'slate', letter: 'D',
    description: 'Render docs → diff against published → review → upload.',
    auto_merge: true, vars_count: 2,
    steps: [
      { id: 'render', type: 'run', after: [] },
      { id: 'diff', type: 'agent', agent: 'doc-reviewer', after: ['render'] },
      { id: 'upload', type: 'run', after: ['diff'] },
    ]
  },
  { id: 'mesh-sync', name: 'mesh-sync', hue: 'green', letter: 'M',
    description: 'Reconcile peer membership; advisory-only, never partitions automatically.',
    auto_merge: false, vars_count: 0,
    steps: [
      { id: 'snapshot', type: 'run', after: [] },
      { id: 'reconcile', type: 'agent', agent: 'mesh-sentinel', after: ['snapshot'] },
    ]
  },
  { id: 'prompt-eval', name: 'prompt-eval', hue: 'pink', letter: 'P',
    description: 'Run the eval set across providers; produce a regression report.',
    auto_merge: false, vars_count: 3,
    steps: [
      { id: 'fixtures', type: 'run', after: [] },
      { id: 'run-eval', type: 'agent', agent: 'prompt-tuner', after: ['fixtures'] },
      { id: 'report', type: 'agent', agent: 'prompt-tuner', after: ['run-eval'] },
    ]
  },
];

// ───────────────── THEMES ─────────────────
const THEMES = [
  { id: 'ship-light',  name: 'ship-light',  hue: 'orange', letter: '◐', kind: 'light',
    description: 'Default. Warm off-white, amber primary.',
    overrides: 12, swatches: ['#f6f4ef', '#d97757', '#3d3a32', '#7e7a72'], dark: false },
  { id: 'ship-dark',   name: 'ship-dark',   hue: 'slate', letter: '◑', kind: 'dark',
    description: 'Warm-navy + amber sibling to ship-light.',
    overrides: 14, swatches: ['#1e2230', '#d97757', '#e8e6e1', '#a09b8f'], dark: true },
  { id: 'paper',       name: 'paper',       hue: 'amber', letter: 'p', kind: 'light',
    description: 'Cream + terracotta. Heavier body, slow motion.',
    overrides: 19, swatches: ['#f1ebe0', '#c0573e', '#2a2520', '#7a6f60'], dark: false },
  { id: 'halo',        name: 'halo',        hue: 'blue', letter: 'h', kind: 'light',
    description: 'Pale-blue neumorph. Depth from shadow, not borders.',
    overrides: 22, swatches: ['#eaf0fa', '#4f7fff', '#1e2640', '#6b7796'], dark: false },
  { id: 'console',     name: 'console',     hue: 'green', letter: '_', kind: 'dark',
    description: 'Phosphor-green on near-black. Mono everywhere. 2px corners.',
    overrides: 28, swatches: ['#0a0e0a', '#7cf38c', '#c5e3ca', '#4f7e57'], dark: true },
  { id: 'contrast',    name: 'contrast',    hue: 'red', letter: '!', kind: 'dark',
    description: 'Near-black + amber fused. 1–2px ring shadows. Bold weights.',
    overrides: 17, swatches: ['#0c0c0e', '#f5a623', '#fafafa', '#a8a8a8'], dark: true },
  { id: 'glass',       name: 'glass',       hue: 'pink', letter: 'g', kind: 'light',
    description: 'Frosted aero. backdrop-filter on every elevated surface.',
    overrides: 24, swatches: ['#f6f0f4', '#c25a8e', '#2e2434', '#8b7080'], dark: false },
];

// ───────────────── COMPONENTS ─────────────────
const COMPONENT_GROUPS = [
  { group: 'Actions',    items: ['sui-button', 'sui-toolbar', 'sui-segmented', 'sui-toggle-group'] },
  { group: 'Inputs',     items: ['sui-input', 'sui-textarea', 'sui-select', 'sui-checkbox', 'sui-radio-group', 'sui-switch', 'sui-slider', 'sui-file-upload', 'sui-field'] },
  { group: 'Layout',     items: ['sui-card', 'sui-divider', 'sui-separator', 'sui-tabs', 'sui-accordion', 'sui-scroll-area', 'sui-collapsible'] },
  { group: 'Feedback',   items: ['sui-alert', 'sui-toast', 'sui-progress', 'sui-spinner', 'sui-skeleton', 'sui-empty-state', 'sui-error-state'] },
  { group: 'Indicators', items: ['sui-badge', 'sui-chip', 'sui-status-dot', 'sui-stat', 'sui-avatar', 'sui-app-icon'] },
  { group: 'Overlays',   items: ['sui-dialog', 'sui-alert-dialog', 'sui-drawer', 'sui-popover', 'sui-tooltip', 'sui-hover-card', 'sui-dropdown-menu', 'sui-context-menu'] },
  { group: 'Data',       items: ['sui-table', 'sui-list-tile', 'sui-tree-view', 'sui-json-view', 'sui-diff', 'sui-timeline', 'sui-history-row'] },
];
const SELECTED_COMPONENT = {
  tag: 'sui-button', group: 'Actions',
  desc: 'Primary action surface. Variant × tone × size × state.',
  props: [
    { name: 'variant', type: "'solid'|'soft'|'outline'|'ghost'|'default'", default: "'default'" },
    { name: 'tone', type: "'primary'|'destructive'|'success'|'info'|'warning'|'accent'", default: 'undefined' },
    { name: 'size', type: "'sm'|'md'|'lg'", default: "'md'" },
    { name: 'disabled', type: 'boolean', default: 'false' },
    { name: 'loading', type: 'boolean', default: 'false' },
  ],
};

// ───────────────── PERMISSIONS ─────────────────
const PERMISSIONS = [
  { id: 'project-read', name: 'project-read', hue: 'blue', letter: 'r',
    description: 'Read-only on the project. Safe for review/inspect agents.',
    allow: ['Read', 'Glob', 'Grep'], ask: ['Bash(git log)', 'Bash(git diff *)'], deny: ['Write', 'Edit', 'Bash(rm *)'],
    used_by: 2 },
  { id: 'project-write', name: 'project-write', hue: 'amber', letter: 'w',
    description: 'Read + write within the project; ask for shell side-effects.',
    allow: ['Read', 'Edit', 'Write', 'Glob', 'Grep', 'Bash(cargo *)', 'Bash(git *)'],
    ask: ['Bash(npm *)', 'Bash(rm *)', 'mcp__github__merge*'],
    deny: ['Bash(rm -rf /*)', 'mcp__*__delete*'], used_by: 5 },
  { id: 'ops', name: 'ops', hue: 'red', letter: 'o',
    description: 'Operator preset — production reads + advisory writes only.',
    allow: ['Read', 'mcp__github__list*', 'mcp__postgres__select*'],
    ask: ['Bash(kubectl *)', 'mcp__github__create*'],
    deny: ['Write', 'Bash(rm *)', 'mcp__postgres__delete*', 'mcp__postgres__truncate*'],
    used_by: 1 },
  { id: 'dangerous', name: 'dangerous', hue: 'red', letter: 'X',
    description: 'Unrestricted. Use only for one-off escape hatches.',
    allow: ['*'], ask: [], deny: [], used_by: 0 },
];

// ───────────────── WIRE — sample agents.wire() responses ─────────────────
//
// What the agent compiles down to per provider/harness. The SDK exposes
// `agents.wire(args)` as a live SSE source. Below is hand-shaped sample
// data matching the schema's harness_settings union so the Wire tab can
// render without runtime traffic.
const WIRE = {
  'rust-compiler': {
    anthropic: {
      file: '~/.claude/agents/rust-compiler.json',
      model: 'claude-sonnet-4-5',
      tools_preset: 'worker',
      system: 'You are rust-compiler. Compile Rust workspaces, surface clippy\nspans inline, block on warnings.\n\nAlways prefer cargo over rustc directly.',
      tools: ['Read', 'Edit', 'Write', 'Glob', 'Grep', 'Bash(cargo *)', 'Bash(git *)'],
      mcp_servers: ['filesystem', 'github'],
      env: { CARGO_TERM_COLOR: 'always' },
    },
    openai: {
      file: '~/.codex/config.toml',
      model: 'gpt-5-codex',
      tools_preset: 'worker',
      system: 'You are rust-compiler. Compile Rust workspaces, surface clippy\nspans inline, block on warnings.',
      tools: ['Read', 'Edit', 'Write', 'Glob', 'Grep', 'Bash(cargo *)', 'Bash(git *)'],
      mcp_servers: ['filesystem', 'github'],
      env: { CARGO_TERM_COLOR: 'always' },
    },
  },
};

// ───────────────── INVOKE — sample skills.invoke() history ─────────────────
//
// `skills.invoke({ id, vars, workspace_id, session_id })` returns
// { run_id, skill_id, session_id, workspace_id, mode, reused_session }.
// Studio surfaces a fixture form + a local history of past invocations
// authored from this Studio session. Sample data:
const INVOKE_HISTORY = {
  tdd: [
    { run_id: 'r_8b3f2c1d', when: '2m ago',     vars: { target: 'crates/runtime/src/scheduler.rs' },
      reused_session: false, session_id: 's_a1b2c3', artifacts: ['scheduler-test-plan.md'], status: 'ok' },
    { run_id: 'r_5a9c0e87', when: '1h ago',     vars: { target: 'crates/sdk/src/http.rs' },
      reused_session: true,  session_id: 's_a1b2c3', artifacts: ['http-tests.md'], status: 'ok' },
    { run_id: 'r_2e7d4b56', when: 'yesterday',  vars: { target: 'crates/runtime/src/kernel.rs' },
      reused_session: false, session_id: 's_44f7e0', artifacts: [], status: 'cancelled' },
  ],
};
