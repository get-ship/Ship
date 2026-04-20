// Eagerly register every ship-* component the SHIP SET docs site
// needs to render. Unlike the marketing site where we pick a narrow
// set for perf, this site exists to *show every element*, so we pull
// the full elements bundle.
//
// Auth dialog + editors aren't included here — they ride in on
// per-page <script> imports when a component page actually uses them.

import '@ship/set/elements';
