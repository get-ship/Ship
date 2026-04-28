<script>
state items = [
  { id: 1, text: 'buy milk', done: false },
  { id: 2, text: 'walk dog', done: true },
]
state draft = ''

function add() {
  const t = draft().trim()
  if (!t) return
  items.set([...items(), { id: Date.now(), text: t, done: false }])
  draft.set('')
}

function toggle(id) {
  items.set(items().map(it => it.id === id ? { ...it, done: !it.done } : it))
}

function remove(id) {
  items.set(items().filter(it => it.id !== id))
}
</script>

<template>
<div>
  <input data-testid="draft" bind:value={draft} on:keydown|key:Enter={add} />
  <button data-testid="add" on:click={add}>add</button>
  <ul data-testid="list">
    {#for it of items() key={it.id}}
      <li data-testid="item">
        <button data-testid="toggle" on:click={() => toggle(it.id)}>{it.done ? 'X' : 'O'}</button>
        <span data-testid="text">{it.text}</span>
        <button data-testid="remove" on:click={() => remove(it.id)}>delete</button>
      </li>
    {:empty}
      <li data-testid="empty">no items</li>
    {/for}
  </ul>
  <p data-testid="count">{items().length} item(s)</p>
</div>
</template>
