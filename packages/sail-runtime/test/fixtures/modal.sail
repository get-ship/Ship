<script>
prop open: boolean = false
function close() { open.set(false) }
function onKey(e) {
  if (e.key === 'Escape') close()
}
</script>

<template>
<div class="modal-root" data-open={open ? 'true' : 'false'}>
  <div class="modal-backdrop" on:click={close} data-testid="backdrop">
    <div class="modal" on:click|stopPropagation={() => {}} data-testid="modal">
      <header data-testid="header">
        <slot name="header"><h3>Dialog</h3></slot>
        <button on:click={close} data-testid="close">×</button>
      </header>
      <div class="body" data-testid="body">
        <slot></slot>
      </div>
      <footer data-testid="footer">
        <slot name="footer"></slot>
      </footer>
    </div>
  </div>
  <sail:document on:keydown={onKey} />
</div>
</template>

<style>
.modal-root[data-open="false"] { display: none; }
.modal-backdrop {
  position: fixed; inset: 0; background: rgba(0,0,0,0.5);
  display: flex; align-items: center; justify-content: center;
}
.modal { background: white; padding: 1rem; min-width: 300px; }
</style>
