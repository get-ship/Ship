import { suiComponentVitestConfig } from '@ship/test-setup/vitest'

// happy-dom: these components inject modern scoped CSS on mount, which jsdom
// 25's CSS parser rejects ("Could not parse CSS stylesheet").
export default suiComponentVitestConfig({ environment: 'happy-dom' })
