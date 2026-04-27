<script>
prop tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
prop variant: 'solid' | 'soft' | 'outline' | 'ghost' = 'solid'
prop size: 'sm' | 'md' | 'lg' = 'md'
prop disabled: boolean = false
prop loading: boolean = false
prop type: 'button' | 'submit' | 'reset' = 'button'
prop label: string = ''
prop pressed: string = ''
prop expanded: string = ''

derived isInert = disabled() || loading()
derived ariaBusy = loading() ? 'true' : 'false'
derived ariaDisabled = isInert() ? 'true' : 'false'
</script>

<template>
<button
  part="button"
  type={type}
  disabled={isInert}
  aria-busy={ariaBusy}
  aria-disabled={ariaDisabled}
  aria-label={label}
  aria-pressed={pressed}
  aria-expanded={expanded}
>
  <span class="content">
    <slot name="leading"></slot>
    <slot></slot>
    <slot name="trailing"></slot>
  </span>
  {#if loading}
    <span class="spinner-wrap" aria-hidden="true">
      <span class="spinner"></span>
    </span>
  {/if}
</button>
</template>

<style>
:host {
  display: inline-flex;
  vertical-align: middle;
  --tone: var(--primary);
  --tone-fg: var(--primary-foreground);
  --tone-contrast: var(--primary-foreground);
}
:host([hidden]) { display: none; }

:host(:not([tone])), :host([tone='default']) {
  --tone: var(--foreground); --tone-fg: var(--foreground); --tone-contrast: var(--background);
}
:host([tone='primary']) {
  --tone: var(--primary); --tone-fg: var(--primary); --tone-contrast: var(--primary-foreground);
}
:host([tone='info']) {
  --tone: var(--info); --tone-fg: var(--info); --tone-contrast: var(--info-foreground);
}
:host([tone='success']) {
  --tone: var(--success); --tone-fg: var(--success); --tone-contrast: var(--success-foreground);
}
:host([tone='warning']) {
  --tone: var(--warning); --tone-fg: var(--warning); --tone-contrast: var(--warning-foreground);
}
:host([tone='destructive']) {
  --tone: var(--destructive); --tone-fg: var(--destructive); --tone-contrast: var(--destructive-foreground);
}
:host([tone='accent']) {
  --tone: var(--accent); --tone-fg: var(--accent); --tone-contrast: var(--accent-foreground);
}
:host([tone='muted']) {
  --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background);
}

button {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--btn-gap);
  font-family: var(--font-body);
  font-weight: var(--btn-fw);
  font-size: var(--btn-fs);
  line-height: 1;
  border-radius: var(--radius-md);
  cursor: pointer;
  border: 1px solid transparent;
  background: transparent;
  color: var(--foreground);
  min-height: var(--btn-h);
  padding-inline: var(--btn-px);
  transition:
    background-color var(--duration-fast) var(--ease-out),
    border-color var(--duration-fast) var(--ease-out),
    box-shadow var(--duration-fast) var(--ease-out),
    color var(--duration-fast) var(--ease-out),
    transform var(--duration-fast) var(--ease-out);
  white-space: nowrap;
  user-select: none;
}

:host(:not([size])), :host([size='md']) {
  --btn-h: 34px; --btn-px: var(--space-3); --btn-fs: var(--text-base);
  --btn-fw: var(--font-weight-semibold); --btn-gap: var(--space-2); --btn-icon: 14px;
}
:host([size='sm']) {
  --btn-h: 28px; --btn-px: var(--space-3); --btn-fs: var(--text-sm);
  --btn-fw: var(--font-weight-medium); --btn-gap: 6px; --btn-icon: 12px;
}
:host([size='lg']) {
  --btn-h: 40px; --btn-px: var(--space-4); --btn-fs: var(--text-md);
  --btn-fw: var(--font-weight-semibold); --btn-gap: var(--space-2); --btn-icon: 16px;
}

:host([variant='solid']) button {
  background: var(--tone);
  border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
  color: var(--tone-contrast);
  box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent), var(--shadow-xs);
}
:host([variant='solid']) button:hover:not(:disabled) {
  background: color-mix(in oklch, var(--tone) 92%, white 8%);
  transform: translateY(-1px);
}

:host([variant='soft']) button {
  background: color-mix(in oklch, var(--tone) 14%, transparent);
  border-color: color-mix(in oklch, var(--tone) 22%, transparent);
  color: var(--tone-fg);
}
:host([variant='outline']) button {
  background: transparent;
  border-color: color-mix(in oklch, var(--tone) 55%, transparent);
  color: var(--tone-fg);
}
:host([variant='ghost']) button {
  background: transparent;
  border-color: transparent;
  color: var(--tone-fg);
}
:host([variant='ghost']) button:hover:not(:disabled) {
  background: color-mix(in oklch, var(--tone) 12%, transparent);
}

button:disabled {
  cursor: not-allowed;
  color: color-mix(in oklch, var(--muted-foreground) 65%, transparent);
  background: var(--muted);
  border-color: color-mix(in oklch, var(--border) 80%, transparent);
  box-shadow: none;
  filter: saturate(0.6);
}

:host([loading]) button { cursor: progress; }
:host([loading]) .content { opacity: 0; }
.content {
  display: inline-flex;
  align-items: center;
  gap: var(--btn-gap);
  transition: opacity var(--duration-fast) var(--ease-out);
}

.spinner-wrap {
  position: absolute;
  inset: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}
.spinner {
  width: var(--btn-icon);
  height: var(--btn-icon);
  border: 1.5px solid currentColor;
  border-right-color: transparent;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  display: inline-block;
}
@keyframes spin { to { transform: rotate(360deg); } }

button:focus-visible {
  outline: none;
  box-shadow:
    0 0 0 var(--ring-offset) var(--background),
    0 0 0 calc(var(--ring-offset) + var(--ring-width))
      color-mix(in oklch, var(--tone) 55%, transparent);
}
</style>
