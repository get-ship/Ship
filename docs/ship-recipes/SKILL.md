---
name: ship-recipes
stable-id: ship-recipes
description: "Idiomatic Ship view recipes — paginated list, optimistic edit, modal-create, tab-synced URL, debounced search, etc. Load when the user asks 'how do I do X in a Ship view'."
tags: [ship, recipes, patterns, views, examples]
authors: [ship]
---

# Ship Recipes

Short, copy-ready patterns for building Ship views. Every recipe is one
page, one pattern, one complete example. Every example uses Ship
primitives only: zero raw Datastar, zero raw CSS. Load the `ship-sdk`
skill for the primitive reference first; come here for "how do I
actually wire X."

## Index

- [Filter chip row](./references/docs/filter-chip-row.md) — faceted list filtering with `chip` + `search`.
- [Tabs synced to URL](./references/docs/tabs-url-sync.md) — `tab` + `tab_panel` round-tripped through `location.hash`.
- [Debounced search](./references/docs/debounced-search.md) — the `search` macro pushing into an SSE list.
- [Two-way bound input](./references/docs/two-way-bound-input.md) — `signal` + `bind` + `data-text`.
- [Form with auto-fields](./references/docs/form-auto-fields.md) — the `form()` macro from a collection schema.
- [Delete with confirm](./references/docs/delete-with-confirm.md) — `confirm_dialog` + `confirm_trigger`.
- [Optimistic update](./references/docs/optimistic-update.md) — `bind` + `put_on_change`.
- [Create + inline-append](./references/docs/create-inline-append.md) — `create` + a signals-patch listener.
- [Loading indicator](./references/docs/loading-indicator.md) — `loading` + `spinner` around an action.
- [Toast after action](./references/docs/toast-after-action.md) — `toast_container` + `toast_action`.
- [Fetch-error surfacing](./references/docs/fetch-error-surfacing.md) — `on_fetch_error` into a `<ship-alert>`.
- [Modal create](./references/docs/modal-create.md) — `confirm_dialog` as a create form host.
- [Two-pane detail view](./references/docs/two-pane-detail.md) — signal-driven selection across a `layout_sidebar`.
- [Infinite-scroll list](./references/docs/infinite-scroll.md) — `data-on:intersect` as the sanctioned raw-Datastar escape.
- [Command palette entry](./references/docs/command-palette-entry.md) — `on('keydown.meta.k')` dispatching a CustomEvent.
