<controller>
import { controller } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-activity-bar.sail'

const CONTEXTS = [
  'agents', 'views', 'components', 'pipelines',
  'themes', 'collections', 'artifacts', 'settings',
] as const
type Context = (typeof CONTEXTS)[number]

function isContext(v: string | undefined): v is Context {
  return v != null && (CONTEXTS as readonly string[]).includes(v)
}

// Vertical context switcher. Each item IS a context; clicking one becomes the
// active context (reflected for styling) and emits ship-context-change so the
// host can swap the navigator panel.
export default controller(({ active }: ViewScope<Scope>, host) => {

    function pick(e: Event): void {
      const ctx = (e.currentTarget as HTMLElement | null)?.dataset.ctx
      if (!isContext(ctx)) return
      active.set(ctx)
      host?.dispatchEvent(
        new CustomEvent('ship-context-change', {
          detail: { context: ctx },
          bubbles: true,
          composed: true,
        }),
      )
    }
    return { pick }
})
</controller>

<!--
  ship-activity-bar.sail — @ship/chrome · the context switcher (vertical). Each item IS a
  context — Agents · Views · Components · Pipelines · Themes — the configurable domains of a
  space. NOT a generic "Explorer", and Library is NOT here (it's a separate surface).
  Selecting a context drives what the navigator panel shows (tree or list). A settings cog
  sits at the foot. Items are the baked set. Each exposes the `item` part for host styling.
  Prop: active(agents|views|components|pipelines|themes|collections|artifacts|settings).
  Controller emits ship-context-change {context} on item click.
-->
<!-- <prop name="active" values="agents|views|components|pipelines|themes|collections|artifacts|settings" default="views" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<button type="button" title="Agents" data-ctx="agents" on:click={pick}><span></span><icon name="bot" size="md"></icon></button>
<button type="button" title="Views" data-ctx="views" on:click={pick}><span></span><icon name="columns" size="md"></icon></button>
<button type="button" title="Components" data-ctx="components" on:click={pick}><span></span><icon name="layout-grid" size="md"></icon></button>
<button type="button" title="Pipelines" data-ctx="pipelines" on:click={pick}><span></span><icon name="workflow" size="md"></icon></button>
<button type="button" title="Themes" data-ctx="themes" on:click={pick}><span></span><icon name="palette" size="md"></icon></button>
<button type="button" title="Collections" data-ctx="collections" on:click={pick}><span></span><icon name="database" size="md"></icon></button>
<button type="button" title="Artifacts" data-ctx="artifacts" on:click={pick}><span></span><icon name="box" size="md"></icon></button>
<span></span>
<button type="button" title="Space settings" data-ctx="settings" on:click={pick}><span></span><icon name="sliders-horizontal" size="md"></icon></button>
