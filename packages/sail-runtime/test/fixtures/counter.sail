<script>
state count = 0
derived doubled = count() * 2
function inc() { count.set(count() + 1) }
function reset() { count.set(0) }
</script>

<template>
<div>
  <p data-testid="count">{count}</p>
  <p data-testid="doubled">{doubled}</p>
  <button data-testid="inc" on:click={inc}>+1</button>
  <button data-testid="reset" on:click={reset}>reset</button>
</div>
</template>
