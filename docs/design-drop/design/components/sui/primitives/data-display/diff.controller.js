// diff.controller.js — runtime twin of sui-diff.controller.ts. The source uses
// bindHtml() (its escape hatch) to paint the rows; here we expose the parsed
// rows as a DERIVED signal and let the .sui iterate them with {#for} — the
// idiomatic SUI path, identical output, fully declarative DOM. The LCS line
// diff is self-contained pure JS (no daemon), so it runs live.
import { controller, derived } from '@ship/sui-runtime';

function diffLines(a, b) {
  const A = a.split('\n');
  const B = b.split('\n');
  const n = A.length;
  const m = B.length;
  const dp = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i][j] = A[i] === B[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
    }
  }
  const out = [];
  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (A[i] === B[j]) { out.push({ kind: 'ctx', text: A[i] }); i++; j++; }
    else if (dp[i + 1][j] >= dp[i][j + 1]) { out.push({ kind: 'del', text: A[i] }); i++; }
    else { out.push({ kind: 'add', text: B[j] }); j++; }
  }
  while (i < n) out.push({ kind: 'del', text: A[i++] });
  while (j < m) out.push({ kind: 'add', text: B[j++] });
  return out;
}

function parseUnified(value) {
  return value.split('\n').map((line) => {
    if (line.startsWith('@@')) return { kind: 'hunk', text: line };
    if (line.startsWith('+')) return { kind: 'add', text: line.slice(1) };
    if (line.startsWith('-')) return { kind: 'del', text: line.slice(1) };
    return { kind: 'ctx', text: line.startsWith(' ') ? line.slice(1) : line };
  });
}

const SIGN = { add: '+', del: '-', hunk: '@', ctx: ' ' };
const LABEL = { add: 'added', del: 'removed', hunk: 'hunk', ctx: 'context' };

export default controller({
  scope: ({ value, from, to }) => {
    const lines = derived(() => {
      const v = value();
      let raw;
      if (v) raw = parseUnified(v);
      else {
        const f = from();
        const t = to();
        raw = (f || t) ? diffLines(f, t) : [];
      }
      return raw.map((l, idx) => ({
        idx,
        kind: l.kind,
        cls: 'line ' + l.kind,
        sign: SIGN[l.kind],
        text: l.text,
        label: LABEL[l.kind] + ': ' + l.text,
      }));
    });
    return { lines };
  },

  mount(_state, host) {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'region');
    if (!host.hasAttribute('aria-label')) host.setAttribute('aria-label', 'diff');
    return () => {};
  },
});
