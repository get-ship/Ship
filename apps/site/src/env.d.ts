/// <reference path="../.astro/types.d.ts" />

declare namespace JSX {
  interface IntrinsicElements {
    [elemName: string]: any;
  }
}

// `@ship/arcade/bundle` is a side-effect-only bundle that registers the
// arcade custom elements (`sui-stage`, sprite, collider, etc.) on import.
// The package emits the JS via `sui-build` but no companion .d.ts. Declare
// the module here so dynamic imports type-check.
declare module '@ship/arcade/bundle';
