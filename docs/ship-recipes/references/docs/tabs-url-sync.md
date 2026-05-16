---
group: Recipes
title: Tabs synced to URL
order: 3
description: Tab state that survives refresh and back/forward via location.hash, using only tab + tab_panel and on('load').
---

When you need tabs whose selection persists across reload and is
shareable as a URL, back the active-tab signal with `location.hash`.
Read the hash on load into the signal; write the hash back whenever
the signal changes. Both sides use the `on()` macro — no inline
listeners.

```jinja
{% from '_ship.html.j2' import signal, on, tab, tab_panel %}

<div class="ship-view"
     {{ signal('activeTab', 'details') }}
     {{ on('load', "$activeTab = (location.hash || '#details').slice(1)") }}>
  <sui-tabs {{ on('ship-change', "location.hash = '#' + $activeTab") }}>
    <sui-tab {{ tab('activeTab', 'details') }}>Details</sui-tab>
    <sui-tab {{ tab('activeTab', 'history') }}>History</sui-tab>
    <sui-tab {{ tab('activeTab', 'logs') }}>Logs</sui-tab>
  </sui-tabs>

  <div {{ tab_panel('activeTab', 'details') }}>...details...</div>
  <div {{ tab_panel('activeTab', 'history') }}>...history...</div>
  <div {{ tab_panel('activeTab', 'logs') }}>...logs...</div>
</div>
```

**Primitives used:** `signal`, `on`, `tab`, `tab_panel`.

**When NOT to use:** if the tab content is server-side expensive, use
separate views and `navigate()` instead — the hash approach keeps all
panels in the DOM.
