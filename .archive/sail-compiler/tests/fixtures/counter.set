<script>
state count = 0
derived doubled = count() * 2
function inc() { count.set(count() + 1) }
function reset() { count.set(0) }
</script>

<template>
<div class="counter">
  <p>Count: {count}</p>
  <p>Doubled: {doubled}</p>
  <button on:click={inc}>+1</button>
  <button on:click={reset}>reset</button>
</div>
</template>
