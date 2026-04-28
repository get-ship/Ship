<script>
state open = false
function toggle() { open.set(!open()) }
</script>

<template>
<div class="card">
  <button on:click={toggle}>Toggle</button>
  {#if open}<p class="body">Hello</p>{/if}
</div>
</template>

<style>
:host { display: block; padding: ${space.3}; }
.card { background: ${color.surface}; border-radius: 4px; }
.body { color: ${color.fg}; margin-top: 0.5rem; }
</style>
