//! MCP tool: generate a starter SUI view.
//!
//! No filesystem writes — returns the `.sui` page, an optional `controller.ts`,
//! and the `manifest.jsonc` as strings so the caller's Edit/Write tool owns the
//! commit. Output follows the view-directory convention
//! (`views/<name>/{page.sui,controller.ts,manifest.jsonc}`) and the SUI idioms
//! the compiler + sui-lsp expect — keeps authoring consistent across agents.

use crate::requests::ViewScaffoldRequest;

/// Dispatch by shape, return a JSON payload carrying the page source, the
/// optional controller source, the manifest, and suggested write paths.
pub fn view_scaffold(req: ViewScaffoldRequest) -> String {
    let name = req.name.trim();
    if name.is_empty() {
        return "Error: `name` is required".to_string();
    }
    let shape = req
        .shape
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("blank");
    let collection = req.collection.as_deref().unwrap_or("<collection>");

    let needs_collection = matches!(shape, "list" | "list-edit-delete" | "form" | "detail");
    if needs_collection && req.collection.as_deref().map(str::is_empty).unwrap_or(true) {
        return format!("Error: shape `{shape}` requires a `collection` argument");
    }

    let title = title_case(name);

    // (page.sui, Some(controller.ts)) per shape. `blank` is the only
    // shape with no reactive state, so it ships without a controller.
    let (page, controller) = match shape {
        "blank" => (page_blank(name, &title), None),
        "list" => (
            page_list(name, &title, collection),
            Some(controller_list(collection)),
        ),
        "list-edit-delete" => (
            page_list_edit_delete(name, &title, collection),
            Some(controller_list_edit_delete(collection)),
        ),
        "form" => (
            page_form(name, &title, collection),
            Some(controller_form(name, collection)),
        ),
        "detail" => (
            page_detail(name, &title, collection),
            Some(controller_detail(collection)),
        ),
        "tabs" => (page_tabs(name, &title), Some(controller_tabs())),
        other => {
            return format!(
                "Error: unknown shape `{other}`; expected one of blank, list, list-edit-delete, form, detail, tabs"
            );
        }
    };

    let manifest = manifest_for(name, &title);

    let mut payload = serde_json::json!({
        "page": page,
        "manifest": manifest,
        "suggested_paths": {
            "page": format!("views/{name}/page.sui"),
            "manifest": format!("views/{name}/manifest.jsonc"),
        },
    });
    if let Some(ctrl) = controller {
        payload["controller"] = serde_json::Value::String(ctrl);
        payload["suggested_paths"]["controller"] =
            serde_json::Value::String(format!("views/{name}/controller.ts"));
    }
    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
}

/// Upper-case the first character — `notes` → `Notes`. Used for the view
/// title; the author replaces it with a real label.
fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn manifest_for(name: &str, title: &str) -> String {
    // jsonc — route metadata per the view-directory convention. The author
    // fills in description/rail; `path` defaults to `/<name>`.
    format!(
        "// Route metadata for views/{name}/. Replace the title and add a\n\
         // `rail` entry once the surface is real.\n\
         {{\n  \"name\": \"{name}\",\n  \"path\": \"/{name}\",\n  \"title\": \"{title}\",\n  \"layout\": \"default\",\n  \"auth\": \"user\"\n}}\n"
    )
}

fn page_blank(name: &str, title: &str) -> String {
    TPL_BLANK.replace("__NAME__", name).replace("__TITLE__", title)
}

fn page_list(name: &str, title: &str, collection: &str) -> String {
    TPL_LIST
        .replace("__NAME__", name)
        .replace("__TITLE__", title)
        .replace("__COLL__", collection)
}

fn page_list_edit_delete(name: &str, title: &str, collection: &str) -> String {
    TPL_LIST_EDIT_DELETE
        .replace("__NAME__", name)
        .replace("__TITLE__", title)
        .replace("__COLL__", collection)
}

fn page_form(name: &str, title: &str, collection: &str) -> String {
    TPL_FORM
        .replace("__NAME__", name)
        .replace("__TITLE__", title)
        .replace("__COLL__", collection)
}

fn page_detail(name: &str, title: &str, collection: &str) -> String {
    TPL_DETAIL
        .replace("__NAME__", name)
        .replace("__TITLE__", title)
        .replace("__COLL__", collection)
}

fn page_tabs(name: &str, title: &str) -> String {
    TPL_TABS.replace("__NAME__", name).replace("__TITLE__", title)
}

fn controller_list(collection: &str) -> String {
    CTRL_LIST.replace("__COLL__", collection)
}

fn controller_list_edit_delete(collection: &str) -> String {
    CTRL_LIST_EDIT_DELETE.replace("__COLL__", collection)
}

fn controller_form(name: &str, collection: &str) -> String {
    CTRL_FORM.replace("__NAME__", name).replace("__COLL__", collection)
}

fn controller_detail(collection: &str) -> String {
    CTRL_DETAIL.replace("__COLL__", collection)
}

fn controller_tabs() -> String {
    CTRL_TABS.to_string()
}

// ── Page templates (.sui) ────────────────────────────────────────────────
// Placeholders: __NAME__ (dir/route id), __TITLE__ (heading), __COLL__
// (collection id). Idioms mirror the first-party views under `views/`.

const TPL_BLANK: &str = r#"<div class="ship-view view-__NAME__">
  <header class="view-head">
    <h1 class="view-title">__TITLE__</h1>
  </header>

  <sui-empty-state variant="placeholder" size="md">
    <sui-icon slot="icon" name="sparkles"></sui-icon>
    <span slot="title">Nothing here yet</span>
    Scaffolded blank view — replace this block with your real content.
  </sui-empty-state>
</div>
"#;

const TPL_LIST: &str = r#"<div class="ship-view view-__NAME__">
  <header class="view-head">
    <h1 class="view-title">__TITLE__</h1>
    <sui-field label="Search">
      <sui-input bind:value={query} placeholder="Search __COLL__…"></sui-input>
    </sui-field>
  </header>

  {#async items}
    {:loading}
    <sui-spinner aria-label="Loading __COLL__"></sui-spinner>

    {:error err}
    <sui-error-state title="Couldn't load __COLL__">{err.message}</sui-error-state>

    {:ok}
    {#if filtered().length === 0}
      <sui-empty-state variant="default" size="sm">
        <sui-icon slot="icon" name="inbox"></sui-icon>
        <span slot="title">No __COLL__ yet</span>
      </sui-empty-state>
    {:else}
      <ul class="view-list">
        {#for item of filtered() key={item.id}}
          <li class="view-row">
            <a href={'/__NAME__/' + item.id}>{item.data.name ?? item.id}</a>
          </li>
        {/for}
      </ul>
    {/if}
  {/async}
</div>
"#;

const TPL_LIST_EDIT_DELETE: &str = r#"<div class="ship-view view-__NAME__">
  <header class="view-head">
    <h1 class="view-title">__TITLE__</h1>
  </header>

  <sui-card>
    <h3 slot="header">Add to __COLL__</h3>
    <div class="view-add-row">
      <sui-input bind:value={draft} placeholder="Name…"
        on:keydown={(e) => e.key === 'Enter' && addItem()}></sui-input>
      <sui-button tone="primary" variant="solid"
        prop:disabled={busy() || !draft().trim()}
        on:click={addItem}>Add</sui-button>
    </div>
  </sui-card>

  {#if mutationError()}
    <sui-error-state variant="inline" size="sm" title="Operation failed">
      {mutationError()}
      <sui-button slot="action" size="sm" variant="outline"
        on:click={() => mutationError.set('')}>Dismiss</sui-button>
    </sui-error-state>
  {/if}

  {#async items}
    {:loading}
    <sui-spinner aria-label="Loading __COLL__"></sui-spinner>

    {:error err}
    <sui-error-state title="Couldn't load __COLL__">{err.message}</sui-error-state>

    {:ok}
    {#if items().length === 0}
      <sui-empty-state variant="default" size="sm">
        <span slot="title">No __COLL__ yet</span>
      </sui-empty-state>
    {:else}
      <ul class="view-list">
        {#for item of items() key={item.id}}
          <li class="view-row">
            <span class="view-row-label">{item.data.name ?? item.id}</span>
            <sui-button size="sm" variant="outline"
              aria-label={'Delete ' + item.id}
              prop:disabled={removingId() === item.id}
              on:click={() => removeItem(item.id)}>
              {removingId() === item.id ? 'Deleting…' : 'Delete'}
            </sui-button>
          </li>
        {/for}
      </ul>
    {/if}
  {/async}
</div>
"#;

const TPL_FORM: &str = r#"<div class="ship-view view-__NAME__">
  <sui-card>
    <h2 slot="header">__TITLE__</h2>

    {#if submitError()}
      <sui-error-state variant="inline" size="sm" title="Save failed">
        {submitError()}
      </sui-error-state>
    {/if}

    <sui-field label="Name">
      <sui-input bind:value={name} placeholder="Name…"></sui-input>
    </sui-field>
    <sui-field label="Description">
      <sui-input bind:value={description} placeholder="Description…"></sui-input>
    </sui-field>

    <sui-button slot="footer" tone="primary" variant="solid"
      prop:disabled={submitting() || !name().trim()}
      on:click={submit}>
      {submitting() ? 'Saving…' : 'Save'}
    </sui-button>
  </sui-card>
</div>
"#;

const TPL_DETAIL: &str = r#"<div class="ship-view view-__NAME__ view-split">
  <aside class="view-pane view-pane-list">
    {#async items}
      {:loading}
      <sui-spinner aria-label="Loading __COLL__"></sui-spinner>

      {:error err}
      <sui-error-state title="Couldn't load __COLL__">{err.message}</sui-error-state>

      {:ok}
      <ul class="view-list">
        {#for item of items() key={item.id}}
          <li class="view-row" data-active={selected() === item.id ? '' : null}>
            <button type="button" on:click={() => selected.set(item.id)}>
              {item.data.name ?? item.id}
            </button>
          </li>
        {/for}
      </ul>
    {/async}
  </aside>

  <section class="view-pane view-pane-detail">
    {#if active()}
      <sui-card>
        <h2 slot="header">{active().data.name ?? active().id}</h2>
        <p>{active().data.description ?? ''}</p>
      </sui-card>
    {:else}
      <sui-empty-state variant="default" size="sm">
        <span slot="title">Select an item</span>
      </sui-empty-state>
    {/if}
  </section>
</div>

<style>
  .view-split { display: grid; grid-template-columns: 320px 1fr; gap: var(--space-4, 1rem); }
  .view-pane { min-width: 0; }
</style>
"#;

const TPL_TABS: &str = r#"<div class="ship-view view-__NAME__">
  <header class="view-head">
    <h1 class="view-title">__TITLE__</h1>
  </header>

  <sui-tabs variant="underline">
    <sui-tab name="overview" active={tab() === 'overview'} on:click={() => tab.set('overview')}>Overview</sui-tab>
    <sui-tab name="activity" active={tab() === 'activity'} on:click={() => tab.set('activity')}>Activity</sui-tab>
    <sui-tab name="settings" active={tab() === 'settings'} on:click={() => tab.set('settings')}>Settings</sui-tab>
  </sui-tabs>

  {#if tab() === 'overview'}
    <sui-empty-state variant="default" size="sm"><span slot="title">Overview</span></sui-empty-state>
  {:else if tab() === 'activity'}
    <sui-empty-state variant="default" size="sm"><span slot="title">Activity</span></sui-empty-state>
  {:else}
    <sui-empty-state variant="default" size="sm"><span slot="title">Settings</span></sui-empty-state>
  {/if}
</div>
"#;

// ── Controller templates (controller.ts) ──────────────────────────────────

const CTRL_LIST: &str = r#"import { controller, signal, live, derived, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const COLLECTION = '__COLL__'

interface Item { id: string; data: Record<string, unknown> }
interface ItemsState { items: Item[] }

export default controller<Scope>({
  scope: ({ ship }) => {
    const query = signal('')
    // `live()` subscribes the SDK source into a LiveSignal so `{#async items}`
    // and `unwrap(items)` work — the `<data>` tag can't carry the arg object.
    const items = live(() => ship.collections.items({ collection: COLLECTION }))
    return { query, items }
  },
  derived: {
    filtered: ({ items, query }): Item[] => {
      const all = (unwrap(items) as ItemsState | undefined)?.items ?? []
      const q = query().toLowerCase().trim()
      if (!q) return all
      return all.filter((it) =>
        String(it.data.name ?? it.id).toLowerCase().includes(q),
      )
    },
  },
})
"#;

const CTRL_LIST_EDIT_DELETE: &str = r#"import { controller, signal, live } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const COLLECTION = '__COLL__'

export default controller<Scope>({
  scope: ({ ship }) => {
    const draft = signal('')
    const busy = signal(false)
    const removingId = signal('')
    const mutationError = signal('')
    const items = live(() => ship.collections.items({ collection: COLLECTION }))

    async function addItem(): Promise<void> {
      const name = draft().trim()
      if (!name) return
      busy.set(true)
      mutationError.set('')
      try {
        // The live `<data>` subscription receives the new item via a
        // projection patch — no manual refresh needed.
        await ship.collections.create_item({ collection: COLLECTION, data: { name } })
        draft.set('')
      } catch (e) {
        mutationError.set((e as Error).message)
      } finally {
        busy.set(false)
      }
    }

    async function removeItem(itemId: string): Promise<void> {
      removingId.set(itemId)
      mutationError.set('')
      try {
        await ship.collections.delete_item({ collection: COLLECTION, item_id: itemId })
      } catch (e) {
        mutationError.set((e as Error).message)
      } finally {
        removingId.set('')
      }
    }

    return { draft, busy, removingId, mutationError, items, addItem, removeItem }
  },
})
"#;

const CTRL_FORM: &str = r#"import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const COLLECTION = '__COLL__'

export default controller<Scope>({
  scope: ({ ship, navigate }) => {
    const name = signal('')
    const description = signal('')
    const submitting = signal(false)
    const submitError = signal('')

    async function submit(): Promise<void> {
      if (!name().trim()) return
      submitting.set(true)
      submitError.set('')
      try {
        await ship.collections.create_item({
          collection: COLLECTION,
          data: { name: name(), description: description() },
        })
        navigate('/__NAME__')
      } catch (e) {
        submitError.set((e as Error).message)
      } finally {
        submitting.set(false)
      }
    }

    return { name, description, submitting, submitError, submit }
  },
})
"#;

const CTRL_DETAIL: &str = r#"import { controller, signal, live, derived, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const COLLECTION = '__COLL__'

interface Item { id: string; data: Record<string, unknown> }
interface ItemsState { items: Item[] }

export default controller<Scope>({
  scope: ({ ship }) => {
    const selected = signal('')
    const items = live(() => ship.collections.items({ collection: COLLECTION }))
    return { selected, items }
  },
  derived: {
    active: ({ items, selected }): Item | null => {
      const all = (unwrap(items) as ItemsState | undefined)?.items ?? []
      return all.find((it) => it.id === selected()) ?? null
    },
  },
})
"#;

const CTRL_TABS: &str = r#"import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => {
    const tab = signal('overview')
    return { tab }
  },
})
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn req(name: &str, shape: &str, collection: Option<&str>) -> ViewScaffoldRequest {
        ViewScaffoldRequest {
            name: name.to_string(),
            collection: collection.map(|s| s.to_string()),
            shape: Some(shape.to_string()),
        }
    }

    fn parse(out: &str) -> serde_json::Value {
        serde_json::from_str(out).unwrap_or_else(|_| panic!("not JSON: {out}"))
    }

    #[test]
    fn blank_shape_emits_sui_page_no_controller() {
        let v = parse(&view_scaffold(req("notes", "blank", None)));
        let page = v["page"].as_str().unwrap();
        assert!(page.contains("sui-empty-state"));
        assert!(page.contains("ship-view view-notes"));
        // No Jinja idioms survive the migration.
        assert!(!page.contains("ship_view_chrome"));
        assert!(!page.contains("{%"));
        assert_eq!(v["suggested_paths"]["page"], "views/notes/page.sui");
        // blank carries no reactive state — no controller.
        assert!(v["controller"].is_null());
    }

    #[test]
    fn list_edit_delete_requires_collection() {
        let out = view_scaffold(req("tasks", "list-edit-delete", None));
        assert!(out.starts_with("Error:"), "expected error, got: {out}");
    }

    #[test]
    fn list_edit_delete_emits_create_and_delete_calls() {
        let v = parse(&view_scaffold(req("tasks", "list-edit-delete", Some("tasks"))));
        let page = v["page"].as_str().unwrap();
        let ctrl = v["controller"].as_str().unwrap();
        assert!(page.contains("sui-button"));
        assert!(ctrl.contains("create_item({ collection: COLLECTION"));
        assert!(ctrl.contains("delete_item({ collection: COLLECTION"));
        assert!(ctrl.contains("const COLLECTION = 'tasks'"));
        assert_eq!(v["suggested_paths"]["controller"], "views/tasks/controller.ts");
    }

    #[test]
    fn list_fetches_collection_in_controller() {
        let v = parse(&view_scaffold(req("notes", "list", Some("notes"))));
        let page = v["page"].as_str().unwrap();
        let ctrl = v["controller"].as_str().unwrap();
        // The arg'd SDK call lives in the controller (the `<data>` source
        // attribute can't carry an object literal); the page consumes it.
        assert!(ctrl.contains("ship.collections.items({ collection: COLLECTION })"));
        assert!(ctrl.contains("const COLLECTION = 'notes'"));
        assert!(page.contains("{#async items}"));
        assert!(!page.contains("<data"));
    }

    #[test]
    fn unknown_shape_errors() {
        let out = view_scaffold(req("x", "wizard", Some("c")));
        assert!(out.contains("unknown shape"), "got: {out}");
    }

    #[test]
    fn empty_name_errors() {
        let out = view_scaffold(req("", "blank", None));
        assert!(out.starts_with("Error:"));
    }

    #[test]
    fn tabs_does_not_require_collection() {
        let v = parse(&view_scaffold(req("settings", "tabs", None)));
        assert!(v["page"].as_str().unwrap().contains("sui-tabs"));
        // tab state lives in the controller.
        assert!(v["controller"].as_str().unwrap().contains("signal('overview')"));
    }

    #[test]
    fn manifest_uses_route_convention() {
        let v = parse(&view_scaffold(req("notes", "blank", None)));
        let m = v["manifest"].as_str().unwrap();
        assert!(m.contains("\"name\": \"notes\""));
        assert!(m.contains("\"path\": \"/notes\""));
        assert!(m.contains("\"title\": \"Notes\""));
        assert!(m.contains("\"auth\": \"user\""));
        // Old Jinja manifest shape is gone.
        assert!(!m.contains("\"type\": \"html\""));
    }
}
