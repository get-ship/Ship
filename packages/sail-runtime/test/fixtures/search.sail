<script>
prop placeholder: string = 'Type to search…'
state items = ['apple', 'banana', 'blueberry', 'cherry', 'date', 'fig']
state query = ''
state caseSensitive = false
derived filtered = items().filter(it => {
  const q = caseSensitive() ? query() : query().toLowerCase()
  const t = caseSensitive() ? it : it.toLowerCase()
  return q === '' || t.includes(q)
})
derived hasResults = filtered().length > 0
</script>

<template>
<div class="search">
  <input
    bind:value={query}
    placeholder={placeholder}
    data-testid="query"
  />
  <label>
    <input type="checkbox" bind:checked={caseSensitive} data-testid="case" />
    case sensitive
  </label>

  {#if hasResults}
    <ul data-testid="results">
      {#for it of filtered() key={it}}
        <li data-testid="result">{it}</li>
      {/for}
    </ul>
  {:else}
    <p data-testid="empty">no matches for "{query}"</p>
  {/if}
</div>
</template>

<style>
.search { padding: 1rem; }
.search input { display: block; }
</style>
