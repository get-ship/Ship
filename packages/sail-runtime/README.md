# @sail/runtime

The browser-side runtime for the Sail framework. Layer 1 of the stack.

Fine-grained reactive signals, custom-element wrapper, patch-protocol applicator. Compiled `.set` modules import from this package.

**Status:** under construction. v0.1 has the signal engine; v1.0 ships when [03-runtime-architecture.md](../../.artifacts/specs/sail/03-runtime-architecture.md) is implementation-complete.

## API

```ts
import { signal, derived, effect, batch, untrack } from '@sail/runtime'

const $count = signal(0)
const $double = derived(() => $count() * 2)

effect(() => {
  console.log('count is', $count(), 'double is', $double())
})

$count.set(5)
// → 'count is 5 double is 10'

batch(() => {
  $count.set(10)
  $count.set(20)
})
// → 'count is 20 double is 40' (one fire, not two)
```

## Spec

See `.artifacts/specs/sail/03-runtime-architecture.md` for the full design.

## Tests

```
pnpm test
```
