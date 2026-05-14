# Audit — error boundaries in SUI rendering

## Current state

The runtime already catches and logs errors at every async boundary
where a user-supplied callback could throw. Each is a `try { ... }
catch (err) { console.error(...) }` that prevents the throw from
propagating into runtime internals:

| Site | What's caught | Behavior |
|---|---|---|
| `mount.ts:166` | view module's `hydrate(target, ctx)` | log + continue mount |
| `mount.ts:180` | controller `mount()` (sync throw) | log + continue |
| `mount.ts:201` | controller `mount()` async rejection | log only |
| `mount.ts:209` | async-mount disposer when view already disposed | log + continue |
| `mount.ts:247` | controller `unmount()` | log + continue cleanup |
| `mount.ts:254` | cleanups pushed onto the view | log + continue siblings |
| `effect-scope.ts:86` | late `onCleanup` after dispose | log + continue |
| `effect-scope.ts:95` | regular cleanup during dispose | log + continue siblings (reverse order preserved) |
| `signal.ts:218` | `effect` callback body | log + continue |
| `signal.ts:209` | effect cleanup before re-run | log + continue |

This is solid baseline discipline. No exception leaks into the runtime
scheduler; a misbehaving view doesn't crash the rest of the page.

## What's missing

The runtime never surfaces an error to the **user**. A controller that
throws shows a broken half-mounted DOM with no indication — only a
console message that most users never see.

There is **no view-level error boundary primitive**. React has
`<ErrorBoundary>`, Vue has `errorCaptured`, Solid has `<ErrorBoundary>`.
SUI has nothing equivalent. Authoring a view that "renders a fallback
when setup or any binding throws" is not possible today.

## Recommendation

Two pieces, mutually-supportive:

1. **Runtime fallback for setup/hydrate/mount failures.** When any of
   those three throw or reject, `mount()` should replace `target`'s
   contents with a minimal `<div role="alert" data-sui-error>` carrying
   the error message (dev) or generic copy (prod). Optionally call a
   host-supplied `onError(err)` callback from `MountOptions`. Estimated
   ~half day plus tests.

2. **Author-facing `<error-boundary>` primitive in `.sui` templates.**
   Wraps a sub-tree; when any binding inside throws on read or
   re-render, the boundary swaps to a `<fallback>` slot. Mirrors
   Solid's `<ErrorBoundary>` shape. Requires compiler support —
   bigger lift, ~day or two.

Neither lands here. The audit's finding is: baseline catches exist,
but **the user is never told something failed**. That's a real
UX gap and a feature SUI needs before it can claim parity with
React/Solid.

## Test additions worth doing now (without the new primitives)

Pin the existing behavior so we don't lose it accidentally:

- `mount.ts` setup that throws synchronously → mount returns a handle,
  cleanups run on dispose, no leaks (existing reentrance tests touch
  similar ground but not this exact flow).
- Async controller `mount()` returning a rejected Promise → no
  uncaught-rejection warning (we already log).
- `hydrate` that throws → view payload's HTML stays in place; no
  partial DOM tear.

I'd add these in the same commit as the runtime fallback primitive
so the behavior change and its test sit together.

## Decision surfaced

Building these two primitives is a feature commitment. Worth doing
before claiming "Lit/React replacement," but a design call:

- Do we want compiler-emitted `<error-boundary>` or only a runtime
  primitive that controllers wrap themselves?
- Should the runtime fallback be opt-in (host passes `onError`) or
  default-on with a built-in fallback UI?
- Dev vs prod error UX: full stack trace in dev, generic in prod —
  how does the runtime know which?

Defer until someone owns the design.
