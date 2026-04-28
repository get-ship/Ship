<script>
prop title: string = 'Untitled'
prop body: string = ''
prop tags: string[] = []
state expanded = false
function toggle() { expanded.set(!expanded()) }
</script>

<template>
<article class="note">
  <header on:click={toggle}>
    <h2 data-testid="title">{title}</h2>
    <button data-testid="toggle">{expanded() ? 'collapse' : 'expand'}</button>
  </header>
  {#if expanded}
    <div data-testid="body">{@html body}</div>
    {#if tags().length > 0}
      <ul data-testid="tags">
        {#for t of tags() key={t}}
          <li>{t}</li>
        {/for}
      </ul>
    {/if}
  {/if}
</article>
</template>

<style>
.note { border: 1px solid #ddd; padding: 0.75rem; }
.note header { cursor: pointer; display: flex; justify-content: space-between; }
</style>
