# @ship/sui-build

SUI bundling pipeline. Owns the AOT compile phase (sui-compiler → `.compiled.js`) and the bundle phase (Bun or esbuild).

## Contract

```ts
import { run, esbuildAdapter } from '@ship/sui-build'
import type { SuiBuildConfig, AdapterContext } from '@ship/sui-build'

const config: SuiBuildConfig = {
  entries: { main: 'src/index.ts' },
  outdir: 'dist',
  mode: 'production',
}

const ctx: AdapterContext = {
  packageRoot: '/absolute/path/to/package',
  outdir: '/absolute/path/to/package/dist',
}

const result = await run(config, ctx, esbuildAdapter)
```

## CLI

```
bun sui-build [--config sui.config.ts] [--bundler bun|esbuild] [--watch] [--mode development|production]
```

Default config: `sui.config.ts` in cwd. Default bundler: Bun if available, else esbuild.
