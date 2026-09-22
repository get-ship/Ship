import { suiComponentVitestConfig } from '@ship/test-setup/vitest'

// happy-dom: these components inject modern scoped CSS (`:host`, custom props);
// jsdom 25's CSS parser throws "Could not parse CSS stylesheet" on mount.
// happy-dom tolerates it. (Same rationale documented in the config factory.)
export default suiComponentVitestConfig({ environment: 'happy-dom' })
