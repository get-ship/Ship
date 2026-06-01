// sui-viewers engine — `createMimeRegistry()`, the thin assembler. Wires the
// detection slices (detect.ts → registry.ts → descriptor.ts) and returns the
// `MimeRegistry` facade. PURE reactive resolution: imports `@ship/sui-runtime`
// ONLY (transitively, via descriptor.ts). No I/O, no DOM, no `ship.*`/`fetch`/
// `<data>`/`@ship/sdk-client`. Inspects already-resolved inputs (content-type
// string, filename, the first N bytes) and returns a descriptor.
//
// Adding a content type = ONE `register({ test, renderer })` entry; the
// dispatcher never changes. This is the headless-engine-behind-a-thin-shell
// pattern the Table spec proved, reused for content.
//
// Authority: build-spec §"Backbone #1 — createMimeRegistry() + the
// <sui-viewer> dispatcher", §"API sketch (the load-bearing surface)".

import type { Derived, Signal } from '@ship/sui-runtime'
import type {
  CreateMimeRegistryOptions,
  RegistryEntry,
  SourceDescriptor,
  ViewerKind,
  ViewerSource,
} from './types'
import { defaultEntries, resolveSource } from './registry'
import { createDescriptorSlice } from './descriptor'

export interface MimeRegistry {
  /** Reactive: set the source; read the descriptor in the dispatcher. */
  source: Signal<ViewerSource | null>
  /** THE thing `{#match}` branches on. */
  descriptor: Derived<SourceDescriptor>
  kind: Derived<ViewerKind>
  renderer: Derived<string | null>
  /** kind !== 'unsupported'. */
  isSupported: Derived<boolean>

  // mutators — call ONLY from on:* / mount (Cardinal 3).
  setSource(s: ViewerSource | null): void
  /** Extend the table (host can add a renderer). Prepended → wins over
   *  defaults for the same signal tier. Recomputes the reactive descriptor. */
  register(entry: RegistryEntry): void
  /** Pure one-shot used by tests + the descriptor derived. */
  resolve(s: ViewerSource): SourceDescriptor
}

export function createMimeRegistry(opts: CreateMimeRegistryOptions = {}): MimeRegistry {
  const fallbackRenderer = opts.fallbackRenderer ?? null
  const initial: RegistryEntry[] = [...(opts.entries ?? []), ...defaultEntries()]
  const slice = createDescriptorSlice(initial, fallbackRenderer)

  return {
    source: slice.source,
    descriptor: slice.descriptor,
    kind: slice.kind,
    renderer: slice.renderer,
    isSupported: slice.isSupported,

    setSource(s: ViewerSource | null): void {
      slice.source.set(s)
    },

    register(entry: RegistryEntry): void {
      // Prepend so a freshly registered entry wins over a default of the same
      // tier; reassign the signal so the descriptor derived recomputes.
      slice.entries.set([entry, ...slice.entries()])
    },

    resolve(s: ViewerSource): SourceDescriptor {
      return resolveSource(slice.entries(), s, fallbackRenderer)
    },
  }
}
