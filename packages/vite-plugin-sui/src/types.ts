export interface ComponentDetect {
  /** Custom-element tag name. Inferred from the file's directory name when
   *  the dirname matches the file stem (e.g. `sui-button/sui-button.sui`),
   *  unless an explicit override is supplied via `Options.detect`. */
  name: string
  /** Whether the component renders into a shadow root. The compiler always
   *  enables shadow DOM for components today (see crates/sui-compiler), so
   *  this is fixed at `true` until the compiler grows an opt-out. */
  shadow: boolean
}

export type ViteFileMode =
  | { kind: 'view' }
  | { kind: 'component'; meta: ComponentDetect }

export interface Options {
  /** Path to the sui-compiler binary. When omitted, the plugin looks at
   *  `~/.ship/runtime/sui-compiler` first (Step 3 toolchain layout) and
   *  falls back to `target/release/sui-compiler` walking up from the
   *  current vite root for in-repo dev. */
  compilerPath?: string
  /** Override the component-vs-view detection. Receives the absolute path
   *  to the .sui file; return `null` to delegate to the default heuristic.
   *  The default treats `<dir>/<dir>.sui` as a component named `<dir>`,
   *  and every other `.sui` file as a view. */
  detect?: (absPath: string) => ViteFileMode | null
  /** Skip emitting the `.sui.d.ts` declaration sidecar during transform.
   *  Defaults to `true` because Vite already supplies module types via
   *  the source-of-truth declaration in the package consumer. */
  noDts?: boolean
}
