/** Configuration shape for a SUI build. Consumers author one of these as
 *  `<package>/sui.config.ts` (default export); the sui-build CLI reads it. */
export interface SuiBuildConfig {
  /** Always-loaded bundle entries. Key = output filename (without .js),
   *  value = source file path relative to the config file. */
  entries: Record<string, string>

  /** Code-split bundles loaded on demand. Same shape as `entries`. */
  lazyBundles?: Record<string, string>

  /** Where bundled outputs land. Relative to the package root.
   *  Flat layout — both `entries` and `lazyBundles` write to the same
   *  directory; no subdirectory for lazy. The daemon's importmap
   *  controls when each loads. */
  outdir: string

  /** Build mode. Defaults to 'production'. CLI sets to 'development'
   *  in watch mode. */
  mode?: 'development' | 'production'

  /** Emit source maps. Default: true in development, false in production. */
  sourcemap?: boolean
}

export interface BuildResult {
  outputs: string[]      // absolute paths of emitted files
  warnings: string[]     // non-fatal messages
  durationMs: number     // total wall-clock time
}

/** A SuiBundler adapter — implementations live under src/adapters/. */
export interface SuiBundler {
  readonly name: 'bun' | 'esbuild'
  build(config: SuiBuildConfig, ctx: AdapterContext): Promise<BuildResult>
}

export interface AdapterContext {
  /** Absolute path to the package directory containing sui.config.ts. */
  packageRoot: string
  /** Absolute path to the resolved outdir. */
  outdir: string
}
