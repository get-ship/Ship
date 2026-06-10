import {
  suiComponentVitestConfig,
  pinImportMetaUrlPlugin,
  shadowForComponentCompilePlugin,
} from '@ship/test-setup/vitest'

// happy-dom (not jsdom): the dispatcher composes `@ship/sui-primitives`
// (sui-markdown, sui-code, sui-json-view, sui-empty-state) which inject modern
// scoped CSS; jsdom 25's parser throws "Could not parse CSS stylesheet" on
// mount, happy-dom tolerates it. The default include globs
// (`test/**/*.test.ts` + `src/**/*.test.ts`) pick up the relocated dispatcher
// suite (under test/) and the co-located engine spec (under src/engine/).
const base = suiComponentVitestConfig({ environment: 'happy-dom' })

export default {
  ...base,
  plugins: [
    pinImportMetaUrlPlugin('ship-viewer'),
    shadowForComponentCompilePlugin('ship-viewer'),
    ...(base.plugins ?? []),
  ],
}
