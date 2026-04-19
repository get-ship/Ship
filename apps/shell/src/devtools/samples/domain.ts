/**
 * Domain / Ship-specific component samples: pipelines, specs, history,
 * events, logs, git-diff, annotations.
 */

import { row, type Sample } from './types'

export const DOMAIN: Record<string, Sample[]> = {
  'ship-pipeline-dag': [
    {
      innerHTML:
        '<ship-pipeline-step name="tests" step-type="agent" state="completed" agent="test-writer"></ship-pipeline-step>' +
        '<span class="arrow">→</span>' +
        '<ship-pipeline-step name="impl" step-type="agent" state="running" agent="rust-lane"></ship-pipeline-step>' +
        '<span class="arrow">→</span>' +
        '<ship-pipeline-step name="gate" step-type="gate" state="pending"></ship-pipeline-step>' +
        '<span class="arrow">→</span>' +
        '<ship-pipeline-step name="merge" step-type="merge" state="pending"></ship-pipeline-step>',
    },
  ],
  'ship-pipeline-step': [
    row(
      [
        '<ship-pipeline-step name="tests" step-type="agent" state="completed" agent="test-writer"></ship-pipeline-step>',
        '<ship-pipeline-step name="impl" step-type="agent" state="running" agent="rust-lane"></ship-pipeline-step>',
        '<ship-pipeline-step name="gate" step-type="gate" state="failed"></ship-pipeline-step>',
        '<ship-pipeline-step name="merge" step-type="merge" state="pending"></ship-pipeline-step>',
      ].join(''),
    ),
  ],
  'ship-pipeline-run': [
    {
      props: { slug: 'fix-auth-142', state: 'running', agent: 'rust-lane' },
      innerHTML:
        '<ship-pipeline-dag>' +
        '<ship-pipeline-step name="tests" state="completed" step-type="agent" agent="test-writer"></ship-pipeline-step>' +
        '<span class="arrow">→</span>' +
        '<ship-pipeline-step name="impl" state="running" step-type="agent" agent="rust-lane"></ship-pipeline-step>' +
        '<span class="arrow">→</span>' +
        '<ship-pipeline-step name="gate" state="pending" step-type="gate"></ship-pipeline-step>' +
        '</ship-pipeline-dag>',
    },
  ],
  'ship-pipeline-definition': [
    {
      raw: true,
      innerHTML:
        '<div class="sample-pair">' +
        '<ship-pipeline-definition name="tdd" steps="4" description="Test-driven development — tests first, then implementation, then gate."></ship-pipeline-definition>' +
        '<ship-pipeline-definition name="polish-pass" steps="6" description="Visual polish pass across primitives; gates on a11y."></ship-pipeline-definition>' +
        '</div>',
      metaAttrs: { name: 'tdd|polish-pass' },
    },
  ],
  'ship-spec-row': [
    {
      raw: true,
      innerHTML:
        '<div class="sample-quad">' +
        '<ship-spec-row item-title="auth-tokens" status="draft" pipeline="tdd"></ship-spec-row>' +
        '<ship-spec-row item-title="rework-gallery" status="ready" pipeline="polish" selected></ship-spec-row>' +
        '<ship-spec-row item-title="pipeline-v2" status="completed" pipeline="tdd"></ship-spec-row>' +
        '<ship-spec-row item-title="flaky-test" status="failed" pipeline="tdd"></ship-spec-row>' +
        '</div>',
      metaAttrs: { status: 'draft|ready|completed|failed' },
    },
  ],
  'ship-history-row': [
    {
      innerHTML:
        '<table style="width:100%;border-collapse:collapse"><tbody>' +
        '<ship-history-row state="completed">' +
        '<span slot="slug" class="mono">fix-auth-142</span>' +
        '<span slot="pipeline">tdd</span>' +
        '<ship-badge slot="status" tone="success">completed</ship-badge>' +
        '<span slot="duration">2m 4s</span>' +
        '<span slot="completed">2m ago</span>' +
        '</ship-history-row>' +
        '<ship-history-row state="failed">' +
        '<span slot="slug" class="mono">flaky-test-88</span>' +
        '<span slot="pipeline">tdd</span>' +
        '<ship-badge slot="status" tone="destructive">failed</ship-badge>' +
        '<span slot="duration">58s</span>' +
        '<span slot="completed">11m ago</span>' +
        '</ship-history-row>' +
        '<ship-history-row state="running">' +
        '<span slot="slug" class="mono">polish-gallery-3</span>' +
        '<span slot="pipeline">polish</span>' +
        '<ship-badge slot="status" tone="info">running</ship-badge>' +
        '<span slot="duration">—</span>' +
        '<span slot="completed">just now</span>' +
        '</ship-history-row>' +
        '</tbody></table>',
    },
  ],
  'ship-event-row': [
    {
      props: {
        actor: 'rust-lane',
        verb: 'started',
        target: 'fix-auth-142',
        at: new Date(Date.now() - 120_000).toISOString(),
      },
      innerHTML: '<ship-code>job=fix-auth branch=fix-auth-142</ship-code>',
    },
    {
      props: {
        actor: 'reviewer',
        verb: 'approved',
        target: 'pipeline-v2',
        at: new Date(Date.now() - 5 * 60_000).toISOString(),
      },
    },
  ],
  'ship-log-line': [
    {
      innerHTML:
        '<ship-log-line level="debug" timestamp="12:03:44" source="daemon">handshake established</ship-log-line>' +
        '<ship-log-line level="info" timestamp="12:03:45" source="runtime">starting pipeline fix-auth-142</ship-log-line>' +
        '<ship-log-line level="warn" timestamp="12:03:46" source="gate">slow response (1.2s)</ship-log-line>' +
        '<ship-log-line level="error" timestamp="12:03:47" source="daemon">connection refused: 127.0.0.1:9316</ship-log-line>',
    },
  ],
  'ship-git-diff': [
    {
      props: { files: '1', additions: '3', deletions: '1' },
      // Hunks only render their header; diff lines are siblings of the
      // hunk inside the file slot, not children of it.
      innerHTML:
        '<ship-git-diff-file path="src/main.rs" additions="3" deletions="1">' +
        '<ship-git-diff-hunk header="@@ -1,3 +1,5 @@"></ship-git-diff-hunk>' +
        '<ship-git-diff-line type="ctx" old="1" new="1">fn main() {</ship-git-diff-line>' +
        '<ship-git-diff-line type="del" old="2">    println!("hi");</ship-git-diff-line>' +
        '<ship-git-diff-line type="add" new="2">    let name = "ship";</ship-git-diff-line>' +
        '<ship-git-diff-line type="add" new="3">    println!("hello, {name}");</ship-git-diff-line>' +
        '<ship-git-diff-line type="add" new="4">    println!("ready");</ship-git-diff-line>' +
        '<ship-git-diff-line type="ctx" old="3" new="5">}</ship-git-diff-line>' +
        '</ship-git-diff-file>',
    },
  ],
  'ship-git-diff-line': [
    row(
      [
        '<ship-git-diff-line type="ctx" old="1" new="1">context line</ship-git-diff-line>',
        '<ship-git-diff-line type="add" new="2">added line</ship-git-diff-line>',
        '<ship-git-diff-line type="del" old="2">deleted line</ship-git-diff-line>',
      ].join(''),
    ),
  ],
  'ship-diff': [
    {
      props: {
        from: 'line one\nline two\nline three',
        to: 'line one\nline 2\nline three\nnew line',
      },
      metaAttrs: { from: 'text', to: 'text' },
    },
  ],
  'ship-annotation-layer': [
    {
      props: { 'view-id': 'gallery-demo', mode: 'off' },
      innerHTML:
        '<div style="padding:18px;border:1px dashed var(--border);border-radius:var(--radius-md);color:var(--muted-foreground)">Annotatable surface (mode=off)</div>',
    },
  ],
  'ship-annotation-marker': [
    {
      innerHTML:
        '<div style="position:relative;width:220px;height:80px;background:var(--muted);border-radius:var(--radius-md)">' +
        '<ship-annotation-marker kind="point" x="40" y="30" index="1"></ship-annotation-marker>' +
        '<ship-annotation-marker kind="point" x="140" y="50" index="2" active></ship-annotation-marker>' +
        '</div>',
    },
  ],
  'ship-annotation-list': [
    {
      innerHTML:
        '<div style="padding:8px;color:var(--muted-foreground);font-size:var(--text-sm)">(no annotations)</div>',
    },
  ],
}
