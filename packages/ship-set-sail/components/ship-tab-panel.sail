<script>
prop name: string = ''
prop active: boolean = false

mount {
  if (!el.hasAttribute('role')) el.setAttribute('role', 'tabpanel')
  if (!el.hasAttribute('tabindex')) el.setAttribute('tabindex', '0')
  if (name() && !el.id) el.id = `ship-tabpanel-${name()}`
  if (name()) el.setAttribute('aria-labelledby', `ship-tab-${name()}`)
}
</script>

<template>
<slot></slot>
</template>

<style>
:host { display: none; }
:host([active]) { display: block; }
</style>
