<script>
prop placeholder: string = 'Search…'
prop minChars: number = 2
state query = ''
derived ready = query.length >= minChars()
</script>

<template>
<div class="typeahead">
  <input
    bind:value={query}
    placeholder={placeholder}
    on:input|debounce:200={() => {}}
  />
  <span>ready: {ready}</span>
</div>
</template>
