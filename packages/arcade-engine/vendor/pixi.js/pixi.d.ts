// Ambient module declaration for the vendored Pixi ESM bundle.
//
// Pixi's published .d.ts is split across hundreds of files in lib/; we don't
// vendor the whole tree. Engine internals treat Pixi as opaque (any) and
// expose typed engine surfaces (World, Entity, Stage) to consumers — so
// Pixi types never leak through the package boundary.

declare module './vendor/pixi.js/pixi.mjs' {
  // biome-ignore lint/suspicious/noExplicitAny: vendored runtime, types not bundled
  const PIXI: any
  export = PIXI
}

declare module '../vendor/pixi.js/pixi.mjs' {
  // biome-ignore lint/suspicious/noExplicitAny: vendored runtime
  const PIXI: any
  export = PIXI
}

declare module '../../vendor/pixi.js/pixi.mjs' {
  // biome-ignore lint/suspicious/noExplicitAny: vendored runtime
  const PIXI: any
  export = PIXI
}
