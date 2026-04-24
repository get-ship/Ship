---
title: "Recipes"
description: "Copy-ready Ship view patterns. Every recipe uses Ship primitives only — zero raw Datastar, zero raw CSS."
sidebar:
  label: "Recipes"
  order: 1
---
The recipes library is a set of focused answers to the question "how
do I do X in a Ship view?" Each page delivers one pattern with one
complete, runnable fragment.

Start with [SDK](/reference/sdk/) if you want the primitive reference —
what every macro does and when to reach for it. Come back here when
you know the macro exists but want to see it composed with its
neighbours.

## The rule

Every example on every page uses only macros imported from
`_ship.html.j2` and elements from the `<ship-*>` library. Raw
`data-signals:`, `data-on:*`, inline `<style>`, and inline `<script>`
do not appear. The one exception — `data-on:intersect` for
infinite-scroll — is called out in place.

## The recipes

1. [Filter chip row](./filter-chip-row.md)
2. [Tabs synced to URL](./tabs-url-sync.md)
3. [Debounced search](./debounced-search.md)
4. [Two-way bound input](./two-way-bound-input.md)
5. [Form with auto-fields](./form-auto-fields.md)
6. [Delete with confirm](./delete-with-confirm.md)
7. [Optimistic update](./optimistic-update.md)
8. [Create + inline-append](./create-inline-append.md)
9. [Loading indicator](./loading-indicator.md)
10. [Toast after action](./toast-after-action.md)
11. [Fetch-error surfacing](./fetch-error-surfacing.md)
12. [Modal create](./modal-create.md)
13. [Two-pane detail view](./two-pane-detail.md)
14. [Infinite-scroll list](./infinite-scroll.md)
15. [Command palette entry](./command-palette-entry.md)
