// View-authoring family — code-split chunk of the app bundle, loaded via load-heavy-chrome.
// Side-effect-registers sui-va-source, sui-va-components, sui-va-activity, sui-va-compile,
// and sui-va-annotation-pins; serves edit-mode surfaces and transitively pulls in sui-component-picker.
// sui-component-picker is a catalog browser that previews any primitive via innerHTML template
// strings — the per-tag resolver cannot trace innerHTML, so the full primitives pack is required here.
// This import lands in the edit-mode chunk (not the eager main chunk), so normal shell operation
// stays tree-shaken; chrome components still trace only the primitives they use via @sui-tag imports.
import '@ship/sui-primitives'
import './sui-va-source.compiled.js'
import './sui-va-components.compiled.js'
import './sui-va-activity.compiled.js'
import './sui-va-compile.compiled.js'
import './sui-va-annotation-pins.compiled.js'
