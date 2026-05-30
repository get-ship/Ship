import { fileURLToPath } from 'node:url'
import { suiComponentVitestConfig } from '@ship/test-setup/vitest'

// Shell-specific: stub xterm's WebGL addon under jsdom (no GL context).
export default suiComponentVitestConfig({
  alias: {
    '@xterm/addon-webgl': fileURLToPath(new URL('./test/__mocks__/addon-webgl.ts', import.meta.url)),
  },
})
