<script>
prop value: number = NaN
prop max: number = 100
prop tone: 'default' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'info'
prop size: 'sm' | 'md' = 'md'
prop label: string = ''

derived isDeterminate = !Number.isNaN(value())
derived pct = isDeterminate()
  ? Math.max(0, Math.min(100, (value() / max()) * 100))
  : null
derived ariaNow = isDeterminate() ? String(value()) : ''
derived ariaBusy = isDeterminate() ? 'false' : 'true'
derived barStyle = pct() != null ? `width: ${pct()}%` : ''
</script>

<template>
<div
  class="track"
  role="progressbar"
  aria-valuemin="0"
  aria-valuemax={max}
  aria-valuenow={ariaNow}
  aria-busy={ariaBusy}
  aria-label={label}
>
  <div class="bar" style={barStyle}></div>
</div>
</template>

<style>
:host { display: block; }
.track {
  background: var(--muted);
  border-radius: 999px;
  overflow: hidden;
  height: 6px;
}
:host([size='sm']) .track { height: 3px; }
.bar {
  height: 100%;
  background: var(--tone-fg);
  border-radius: inherit;
  transition: width 180ms ease;
}
:host(:not([value])) .bar {
  width: 35%;
  animation: indet 1.4s cubic-bezier(0.4, 0, 0.2, 1) infinite;
}
@keyframes indet {
  0% { transform: translateX(-100%); }
  60% { transform: translateX(200%); }
  100% { transform: translateX(200%); }
}
</style>
