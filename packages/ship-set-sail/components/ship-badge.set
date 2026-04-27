<script>
prop tone: 'default' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'muted'
prop variant: 'solid' | 'soft' | 'outline' = 'soft'
prop size: 'sm' | 'md' | 'lg' = 'md'
prop pill: boolean = false
</script>

<template>
<span class="badge" part="badge">
  <span class="leading" part="leading"><slot name="leading"></slot></span>
  <slot></slot>
</span>
</template>

<style>
:host {
  display: inline-flex;
  vertical-align: middle;
  --tone: var(--muted-foreground);
  --tone-fg: var(--muted-foreground);
  --tone-contrast: var(--background);
}
:host([hidden]) { display: none; }

:host(:not([tone])), :host([tone='muted']) {
  --tone: var(--muted-foreground);
  --tone-fg: var(--muted-foreground);
  --tone-contrast: var(--background);
}
:host([tone='default']) {
  --tone: var(--foreground);
  --tone-fg: var(--foreground);
  --tone-contrast: var(--background);
}
:host([tone='info']) {
  --tone: var(--info);
  --tone-fg: var(--info);
  --tone-contrast: var(--info-foreground);
}
:host([tone='success']) {
  --tone: var(--success);
  --tone-fg: var(--success);
  --tone-contrast: var(--success-foreground);
}
:host([tone='warning']) {
  --tone: var(--warning);
  --tone-fg: var(--warning);
  --tone-contrast: var(--warning-foreground);
}
:host([tone='destructive']) {
  --tone: var(--destructive);
  --tone-fg: var(--destructive);
  --tone-contrast: var(--destructive-foreground);
}
:host([tone='accent']) {
  --tone: var(--accent);
  --tone-fg: var(--accent);
  --tone-contrast: var(--accent-foreground);
}

:host(:not([size])), :host([size='md']) {
  --bdg-h: 22px; --bdg-px: 8px; --bdg-fs: 10.5px; --bdg-gap: 4px; --bdg-icon: 10px;
}
:host([size='sm']) {
  --bdg-h: 18px; --bdg-px: 6px; --bdg-fs: 9.5px; --bdg-gap: 3px; --bdg-icon: 8px;
}
:host([size='lg']) {
  --bdg-h: 26px; --bdg-px: 10px; --bdg-fs: 11.5px; --bdg-gap: 5px; --bdg-icon: 12px;
}

:host { --bdg-radius: var(--radius-sm); }
:host([pill]) { --bdg-radius: var(--radius-full); }

.badge {
  display: inline-flex;
  align-items: center;
  gap: var(--bdg-gap);
  min-height: var(--bdg-h);
  padding: 0 var(--bdg-px);
  font-family: var(--font-body);
  font-size: var(--bdg-fs);
  font-weight: var(--font-weight-semibold);
  line-height: 1;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  border-radius: var(--bdg-radius);
  background: color-mix(in oklch, var(--tone) 16%, transparent);
  border: 1px solid color-mix(in oklch, var(--tone) 24%, transparent);
  color: var(--tone-fg);
  white-space: nowrap;
  -webkit-font-smoothing: antialiased;
  font-variant-numeric: tabular-nums;
}

:host([variant='soft']) .badge {
  background: color-mix(in oklch, var(--tone) 16%, transparent);
  border-color: color-mix(in oklch, var(--tone) 24%, transparent);
  color: var(--tone-fg);
}
:host([variant='solid']) .badge {
  background: var(--tone);
  border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
  color: var(--tone-contrast);
  box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent);
}
:host([variant='solid'][tone='muted']) .badge,
:host([variant='solid']:not([tone])) .badge {
  background: var(--muted-foreground);
  border-color: color-mix(in oklch, var(--muted-foreground) 100%, black 6%);
  color: var(--background);
}
:host([variant='outline']) .badge {
  background: transparent;
  border-color: color-mix(in oklch, var(--tone) 50%, transparent);
  color: var(--tone-fg);
}
:host([variant='outline']:not([tone])) .badge,
:host([variant='outline'][tone='muted']) .badge {
  border-color: var(--border);
  color: var(--muted-foreground);
}

.leading {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: var(--bdg-icon);
  height: var(--bdg-icon);
  flex-shrink: 0;
  color: currentColor;
}

@media (forced-colors: active) {
  .badge { border-color: CanvasText; color: CanvasText; }
  :host([variant='solid']) .badge {
    background: Highlight;
    color: HighlightText;
    border-color: Highlight;
  }
}
</style>
