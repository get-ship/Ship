/** Configuration shape for a SUI build. Consumers author one of these as
 *  `<package>/sui.config.ts` (default export); the sui-build CLI reads it. */
export interface SuiBuildConfig {
  /** Always-loaded bundle entries. Key = output filename (without .js),
   *  value = source file path relative to the config file. */
  entries: Record<string, string>

  /** Code-split bundles loaded on demand. Same shape as `entries`. */
  lazyBundles?: Record<string, string>

  /** Where bundled outputs land. Relative to the package root. `entries`
   *  emit directly under `outdir`; `lazyBundles` emit under
   *  `outdir/<lazyOutSubdir>` when set (see below). */
  outdir: string

  /** Subdirectory of `outdir` for `lazyBundles` outputs. When omitted,
   *  lazy bundles share `outdir` with `entries`. Used to keep
   *  code-split islands segregated from always-loaded framework bundles
   *  (e.g. `sui-islands/`). */
  lazyOutSubdir?: string

  /** Build mode. Defaults to 'production'. CLI sets to 'development'
   *  in watch mode. */
  mode?: 'development' | 'production'

  /** Emit source maps. Default: true in development, false in production. */
  sourcemap?: boolean

  /** File-type loader overrides. Map of '.ext' → loader name.
   *  Loader semantics follow esbuild's vocabulary: 'js', 'ts', 'jsx', 'tsx',
   *  'css', 'json', 'text', 'binary', 'base64', 'file', 'copy', 'dataurl'.
   *  Adapters must honor these or throw a clear error if unsupported.
   *  Example: { '.svg': 'text' } inlines SVG content as a JS string. */
  loaders?: Record<string, string>

  /** Aggregate per-component `*.custom-elements.json` to this path after a
   *  successful build. Relative to the config file (sui.config.ts). */
  aggregateCemTo?: string

  /** Directories to scan for per-component CEMs. Defaults to the parent dirs
   *  of `entries` + `lazyBundles` source files (i.e. `src/`). Relative to the
   *  config file. */
  aggregateCemSources?: string[]
}

export interface BuildResult {
  outputs: string[]      // absolute paths of emitted files
  warnings: string[]     // non-fatal messages
  durationMs: number     // total wall-clock time
}

/** A SuiBundler adapter — implementations live under src/adapters/. */
export interface SuiBundler {
  readonly name: 'bun'
  build(config: SuiBuildConfig, ctx: AdapterContext): Promise<BuildResult>
}

export interface AdapterContext {
  /** Absolute path to the package directory containing sui.config.ts. */
  packageRoot: string
  /** Absolute path to the resolved outdir. */
  outdir: string
}
