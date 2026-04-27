<script>
prop active: string = ''
prop variant: 'underline' | 'segmented' = 'underline'
prop size: 'sm' | 'md' | 'lg' = 'md'
prop tone: 'default' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
prop sticky: boolean = false
prop orientation: 'horizontal' | 'vertical' = 'horizontal'

state tabs = []

// Sail uses light DOM and re-parents slot fills into the slot's
// container, so :scope > ship-tab returns nothing once the template
// has rendered. Query the whole subtree instead — the panels live
// only inside our default slot, so this stays unambiguous.
function readTabs() {
  return Array.from(el.querySelectorAll('ship-tab')).map((t, idx) => ({
    index: idx,
    name: t.getAttribute('name') || '',
    label: t.getAttribute('label') || t.getAttribute('name') || '',
    count: t.hasAttribute('count') ? Number(t.getAttribute('count')) : null,
    disabled: t.hasAttribute('disabled'),
  }))
}

function syncPanels() {
  const cur = active()
  for (const p of el.querySelectorAll('ship-tab-panel')) {
    if (p.getAttribute('name') === cur) p.setAttribute('active', '')
    else p.removeAttribute('active')
  }
}

function activate(name) {
  if (active() === name) return
  active.set(name)
  // Reflect to attribute so external observers see the change.
  // Sail doesn't yet support `prop … reflect` — manual for now.
  el.setAttribute('active', name)
  el.dispatchEvent(new CustomEvent('ship-tab-change', {
    detail: { name }, bubbles: true, composed: true,
  }))
}

function onKeydown(e) {
  const vertical = orientation() === 'vertical'
  const nextKey = vertical ? 'ArrowDown' : 'ArrowRight'
  const prevKey = vertical ? 'ArrowUp' : 'ArrowLeft'
  if (![nextKey, prevKey, 'Home', 'End'].includes(e.key)) return
  e.preventDefault()
  const enabled = tabs().filter(t => !t.disabled)
  if (!enabled.length) return
  const names = enabled.map(t => t.name)
  const i = Math.max(0, names.indexOf(active()))
  let next = i
  if (e.key === nextKey) next = (i + 1) % names.length
  else if (e.key === prevKey) next = (i - 1 + names.length) % names.length
  else if (e.key === 'Home') next = 0
  else if (e.key === 'End') next = names.length - 1
  activate(names[next])
}

// Underline indicator: measure the active tab's offset within the
// tablist, write geometry into CSS custom props, and let the
// pseudo-element transition animate the move.
function syncIndicator() {
  if (variant() === 'segmented') {
    el.removeAttribute('data-underline-ready')
    return
  }
  const tablist = el.querySelector('.tablist')
  if (!tablist) return
  const activeBtn = tablist.querySelector('button[aria-selected="true"]')
  if (!activeBtn) {
    el.removeAttribute('data-underline-ready')
    return
  }
  if (orientation() === 'vertical') {
    tablist.style.setProperty('--underline-y', `${activeBtn.offsetTop}px`)
    tablist.style.setProperty('--underline-h', `${activeBtn.offsetHeight}px`)
  } else {
    tablist.style.setProperty('--underline-x', `${activeBtn.offsetLeft}px`)
    tablist.style.setProperty('--underline-w', `${activeBtn.offsetWidth}px`)
  }
  el.setAttribute('data-underline-ready', '')
}

mount {
  // Snapshot tab metadata at mount. Dynamic add/remove of <ship-tab>
  // children is not yet supported — observing the host with a
  // MutationObserver here would loop indefinitely because the {#for}
  // renders buttons inside the same subtree, triggering re-observation.
  // Sail v1.5 will introduce a "scoped observer" primitive for this
  // pattern (or the runtime will distinguish self-emitted mutations).
  tabs.set(readTabs())
  syncPanels()
  // Defer the first indicator placement so the {#for}-rendered
  // buttons are in the DOM before we measure. queueMicrotask is
  // sufficient — bindFor's effect runs synchronously, but the
  // browser hasn't yet laid out the inserted nodes.
  queueMicrotask(syncIndicator)
}

effect on active {
  syncPanels()
  queueMicrotask(syncIndicator)
}
</script>

<template>
<div
  class="tablist"
  part="tablist"
  role="tablist"
  aria-orientation={orientation}
  on:keydown={onKeydown}
>
  {#for t of tabs() key={t.name}}
    <button
      class="tab"
      part="tab"
      role="tab"
      id={`ship-tab-${t.name}`}
      data-name={t.name}
      aria-selected={t.name === active() ? 'true' : 'false'}
      aria-controls={`ship-tabpanel-${t.name}`}
      aria-disabled={t.disabled ? 'true' : 'false'}
      tabindex={t.name === active() ? '0' : '-1'}
      disabled={t.disabled}
      on:click={() => !t.disabled && activate(t.name)}
    >
      <span class="label">{t.label}</span>
      {#if t.count != null}
        <span class="count" aria-hidden="true">{t.count}</span>
      {/if}
    </button>
  {/for}
</div>
<div class="panels" part="panels">
  <slot></slot>
</div>
</template>

<style>
:host {
  display: block;
  --tone: var(--foreground);
  --tone-fg: var(--foreground);
}
:host([tone='info']) { --tone: var(--info); --tone-fg: var(--info); }
:host([tone='success']) { --tone: var(--success); --tone-fg: var(--success); }
:host([tone='warning']) { --tone: var(--warning); --tone-fg: var(--warning); }
:host([tone='destructive']) { --tone: var(--destructive); --tone-fg: var(--destructive); }
:host([tone='accent']) { --tone: var(--accent); --tone-fg: var(--accent); }

:host([sticky]) {
  position: sticky;
  top: 0;
  z-index: 1;
  background: var(--background);
}

.tablist {
  display: inline-flex;
  gap: var(--space-1);
  border-bottom: 1px solid var(--border);
  position: relative;
  --underline-x: 0;
  --underline-w: 0;
  --underline-y: 0;
  --underline-h: 0;
}
:host([orientation='vertical']) .tablist {
  flex-direction: column;
  border-bottom: none;
  border-right: 1px solid var(--border);
}

/* Animated underline indicator. Hidden until [data-underline-ready]
 * is set so the first paint doesn't snap from 0,0. */
:host([data-underline-ready]) .tablist::after {
  content: '';
  position: absolute;
  background: var(--tone);
  border-radius: 2px;
  pointer-events: none;
  transition:
    transform 220ms var(--ease-spring, cubic-bezier(0.5, 0, 0.2, 1.4)),
    width 220ms var(--ease-spring, cubic-bezier(0.5, 0, 0.2, 1.4)),
    height 220ms var(--ease-spring, cubic-bezier(0.5, 0, 0.2, 1.4));
}
:host([data-underline-ready]:not([orientation='vertical'])) .tablist::after {
  height: 2px;
  bottom: -1px;
  left: 0;
  width: var(--underline-w);
  transform: translateX(var(--underline-x));
}
:host([data-underline-ready][orientation='vertical']) .tablist::after {
  width: 2px;
  right: -1px;
  top: 0;
  height: var(--underline-h);
  transform: translateY(var(--underline-y));
}
:host([variant='segmented']) .tablist::after { display: none; }
@media (prefers-reduced-motion: reduce) {
  .tablist::after { transition: none; }
}

.tab {
  appearance: none;
  background: transparent;
  border: 0;
  padding: var(--space-2) var(--space-3);
  font-family: var(--font-body);
  color: var(--muted-foreground);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  border-bottom: 2px solid transparent;
}
:host([orientation='vertical']) .tab {
  border-bottom: 0;
  border-right: 2px solid transparent;
}
.tab[aria-selected='true'] {
  color: var(--tone-fg);
}
/* Pre-underline-ready and segmented variants: fall back to a static
 * border so the active tab is visible without the animated indicator. */
:host(:not([data-underline-ready])) .tab[aria-selected='true'],
:host([variant='segmented']) .tab[aria-selected='true'] {
  /* segmented uses a different active treatment below */
}
.tab:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.tablist.segmented {
  background: var(--muted);
  border-radius: var(--radius-md);
  border-bottom: 0;
  padding: 2px;
}
.tablist.segmented .tab {
  border-bottom: 0;
  border-radius: calc(var(--radius-md) - 2px);
}
.tablist.segmented .tab[aria-selected='true'] {
  background: var(--background);
  color: var(--foreground);
  box-shadow: var(--shadow-xs);
}

.count {
  font-size: var(--text-xs);
  background: color-mix(in oklch, var(--tone) 16%, transparent);
  color: var(--tone-fg);
  border-radius: var(--radius-full);
  padding: 0 6px;
  min-width: 1.25em;
  text-align: center;
}

.panels { padding-block-start: var(--space-3); }
</style>
