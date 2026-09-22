// @ship/skills — Ship Skills pack (ship-* tags for the hub /studio/skills surface).
//
// Each <tag>.sail calls customElements.define() on import, so a
// single `import '@ship/skills'` registers every tag globally.
//
// Composites:
//   <ship-skill-list>   the hub `/studio/skills` index. Reads
//                       `ship.skills.list()` (Connect SDK) and renders
//                       the registered skills, grouped by
//                       SkillScopeTag (project/user). Empty/loading/
//                       error states are first-class.
//
//   <ship-skill-detail> the hub `/studio/skills/:id` detail. Reads
//                       `ship.skills.detail({ id })` (Connect SDK) and
//                       renders the frontmatter + body + files. The
//                       `:id` route param flows through the hub route
//                       controller as the `id` prop; the empty-state
//                       renders for unknown ids.
//
// The pack composes <sui-stack>, <sui-card>, <sui-empty-state>,
// <sui-spinner>, <sui-icon>, <sui-badge>, <sui-cluster>,
// <sui-skeleton> from @ship/sail-primitives. No new primitives, no
// forked layout — per the placement hard-rule (charter §Placement).
import './ship-skill-list/ship-skill-list.compiled.js'
import './ship-skill-detail/ship-skill-detail.compiled.js'
