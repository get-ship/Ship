// `<sui-rail>` lazy bundle. The main `@ship/ui-components` aggregate
// loads first and side-effect-imports `@ship/primitives`, which
// registers `<sui-popover>` globally. By the time this lazy bundle
// runs, the primitive is already defined — no explicit import needed.
// `<sui-view-picker>` is a sibling chrome component registered by the
// umbrella aggregate as well; the import here is defensive in case this
// lazy bundle ever loads before the umbrella (currently it doesn't).
import '../sui-view-picker/sui-view-picker.compiled.js'
import './sui-rail.compiled.js'
import './sui-rail-item.compiled.js'
import './sui-rail-section.compiled.js'
