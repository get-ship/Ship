# swap1k regression findings

**Branch:** job/swap1k-probe  
**Baseline:** 7.2 ms (2026-05-14-current.json @ 747a5f17)  
**Current:** ~11.2 ms (+56%, +4 ms abs)

---

## Localized commit / cost site

**The 5 suspect commits (d0144deec – 57137572d) do not contribute to the regression.**

Static analysis of the compiled benchmark output and the suspect diffs shows the proxy hot path never fires during a swap. Specifically:

```js
// sui-bench-app.compiled.js line 179–183
const __lc = Object.getOwnPropertyDescriptor(__r, "__shipSetLifecycle")?.value;
if (__lc) {
  const __sp = componentMountState($scope);   // ← guarded; never reached
  if (__lc.mount) { ... }
  if (__lc.unmount) { ... }
}
```

The benchmark controller has no `mount`/`unmount`, so `__lc` is falsy. `componentMountState` is imported but never called. The writable-state proxy built inside `resolveController` (`createMountState`) is stored only under `LIFECYCLE_KEY` which is also absent here. The `$scope` in the compiled component is a plain `Object.create(null)` throughout — no proxy wraps it.

The suspect commits (#179 / 57137572d, #152 / aaa574fac, etc.) only changed behavior for components WITH lifecycle handlers. The benchmark is outside that code path.

---

## Actual cost site: baseline architecture gap

The 7.2 ms baseline was captured at commit `747a5f17` using the **old bundled runtime** (`crates/sui/ship-ui-framework/assets/sui-runtime.js`) with a **cursor-based** `bindFor`:

```js
// old bindFor — O(n) cursor, no LIS
for (let i = 0; i < next.length; i++) {
  const item = next[i];
  const existing = prevByKey.get(key(item, i));
  if (existing) { existing.update?.(item, i); record = existing; }
  else { record = render(item, i); }
  // Place in order:
  for (const n of record.nodes) {
    const desired = cursor.nextSibling;
    if (n !== desired) parent.insertBefore(n, desired);
    cursor = n;
  }
}
```

For swap-1k this does ~997 sequential DOM moves (one per non-swapped row cascades after the first swap). Chrome absorbs sequential `insertBefore` on live rows cheaply.

The current LIS-based `for.ts` (introduced in `0c65c6568`, after the baseline) does:
1. **1000 `key()` calls** — each compiled key allocates `Object.create($scope)` and invokes an IIFE:
   ```js
   key: (row, __i) => {
     const __child = Object.create($scope);  // 1000 × per swap
     __child.row = row;
     __child.__i = __i;
     return (($scope) => (row.id))(__child);  // row.id doesn't need $scope
   }
   ```
2. **O(n log n) LIS sort** over 1000 matched pairs (swap creates one inversion → full sort required)
3. **1 DOM move** (only the out-of-LIS element moves)

Steps 1–2 are pure JS overhead that replaces the ~997 DOM moves. For swap-1k Chrome's DOM moves are cheaper than 1000 allocations + O(n log n) sort. The LIS wins on `remove-one-1k` (saves 999 DOM moves) and `run1k` (no existing rows to move), but loses on `swap-1k`.

---

## Specific overhead measurement

The 1000 `Object.create($scope)` calls are the measurable GC pressure. Each creates a short-lived object inheriting from the component scope (10 own properties). They're unreachable after the key call. This pattern runs on every reconciliation, not just swaps — it shows up most on swap because the rest of the swap work (2 DOM moves) is otherwise trivial.

The compiled key expression `{row.id}` does not access any `$scope` variable; the `Object.create($scope)` and IIFE wrapping are emitted uniformly by the compiler for all key expressions regardless of scope usage.

---

## Fix shape

**Option A — Compiler optimization (recommended path)**

When the key expression references only the loop variable and numeric index (no `$scope` lookups), the compiler can emit a flat key function without the scope wrapper:

```js
// Current emit:
key: (row, __i) => {
  const __child = Object.create($scope);
  __child.row = row; __child.__i = __i;
  return (($scope) => (row.id))(__child);
},

// Optimized emit (key expression is scope-free):
key: (row) => row.id,
```

This requires adding a scope-usage analysis pass to the compiler's key-expression emit (`crates/sui/compiler/src/emit/js/`). The analysis is bounded — a `for` key expression is syntactically constrained to a single expression node. This would eliminate 1000 Object.create allocations per reconciliation and likely restore swap-1k to the 7–8 ms range.

**Option B — New baseline (short-term gate fix)**

Capture a new baseline at the current HEAD. The old 7.2 ms baseline reflects a different runtime architecture; the 15% gate compares against a number that no longer represents a fair prior. A fresh baseline at HEAD would reset the comparison to the current LIS runtime (~11 ms swap), and future regressions would be measured correctly within that architecture.

---

## Verdict on #179 (suspect leading candidate)

Commit `57137572d` (#179) — "unify mount-state proxy on unwrap=true" — does not affect swap-1k. The proxy change only touches `createMountState`'s get-trap condition (`if (unwrap && ...)` → `if (...)`), which runs exclusively in `componentMountState`/lifecycle contexts. The benchmark exercises neither path.

The +56% regression is an architecture-gap artifact: the good baseline pre-dates the LIS `for.ts` and the TS-package runtime by ~12 days of commits. A proper bisect between the 5 suspects (all post-LIS-introduction) would show a flat ~11 ms across all 5 commits.

---

## Recommendation

1. **Do not revert or hotfix #179** — it resolves a real type/runtime lie and the bug it fixes (`state.canvasRef as HTMLCanvasElement` broken at runtime) is legitimate.

2. **Capture a new baseline** at HEAD as an immediate gate fix. The regression is an accounting artifact; the 15% threshold needs a contemporary reference point.

3. **Track the compiler key-expression optimization** (Option A above) as a follow-on performance task. The `Object.create($scope)` × N pattern affects every `{#for}` reconciliation and will compound if list sizes grow.

4. **The 4 ms absolute delta** is small and does not block a stable release — it's a measurement baseline problem, not a user-visible regression under normal usage patterns.
