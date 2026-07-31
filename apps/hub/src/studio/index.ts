// apps/hub/src/studio — hub-local studio components.
//
// These components are hub-only (ship-component-catalog, ship-component-viewer,
// ship-studio-chrome, ship-studio-rail, ship-view-chrome). They were moved from
// packages/components/ship/studio when it was determined they serve no other app.
//
// Each <tag>.compiled.js calls customElements.define() on import; a single
// import of this entry registers all five tags globally in the hub bundle.
import './ship-component-catalog/ship-component-catalog.compiled.js'
import './ship-component-viewer/ship-component-viewer.compiled.js'
import './ship-studio-chrome/ship-studio-chrome.compiled.js'
import './ship-studio-rail/ship-studio-rail.compiled.js'
import './ship-view-chrome/ship-view-chrome.compiled.js'
