// `<sui-rail>` separate bundle. The shell's `sui-chrome.js` aggregate
// loads first and registers every non-separate chrome tag globally
// (including `<sui-view-picker>`), so this bundle only needs to
// register its own rail/-item/-section tags.

import './sui-rail.compiled.js'
import './sui-rail-item.compiled.js'
import './sui-rail-section.compiled.js'
