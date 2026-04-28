//! Framework manifest — JSON-serialisable description of every
//! directive, scope binding, lifecycle hook, and template-syntax form
//! the compiler enforces. Feeds dispatched-agent prompts, `sail-lsp`
//! completion, and the docs site. Bumps independently of
//! `sdk-ship-ops.json`.

use serde::{Deserialize, Serialize};

pub const MANIFEST_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub version: String,
    pub directives: Vec<Directive>,
    pub scope_bindings: Vec<ScopeBinding>,
    pub lifecycle_hooks: Vec<LifecycleHook>,
    pub template_syntax: Vec<TemplateSyntax>,
    pub reactivity_rules: Vec<ReactivityRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Directive {
    pub name: String,
    pub kind: String,
    pub syntax: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub anti_examples: Vec<AntiExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AntiExample { pub code: String, pub reason: String }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScopeBinding {
    pub name: String,
    #[serde(rename = "type")] pub ty: String,
    pub available_in: Vec<String>, pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LifecycleHook { pub name: String, pub syntax: String, pub fires: String, pub description: String }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemplateSyntax {
    pub name: String, pub syntax: String, pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReactivityRule { pub id: String, pub name: String, pub description: String, pub anti_example: String, pub fix_example: String }

/// Build the framework manifest. Pure — no I/O, deterministic.
pub fn emit() -> Manifest {
    Manifest {
        version: MANIFEST_VERSION.into(),
        directives: directives(),
        scope_bindings: scope_bindings(),
        lifecycle_hooks: lifecycle_hooks(),
        template_syntax: template_syntax(),
        reactivity_rules: reactivity_rules(),
    }
}

fn vs(xs: &[&str]) -> Vec<String> { xs.iter().map(|s| (*s).into()).collect() }

fn dir(name: &str, syntax: &str, desc: &str, ex: &[&str], anti: &[(&str, &str)]) -> Directive {
    Directive {
        name: name.into(), kind: "script_decl".into(),
        syntax: syntax.into(), description: desc.into(), examples: vs(ex),
        anti_examples: anti.iter()
            .map(|(c, r)| AntiExample { code: (*c).into(), reason: (*r).into() }).collect(),
    }
}

fn directives() -> Vec<Directive> {
    vec![
        dir("prop", "prop NAME [: TYPE] [!] = DEFAULT",
            "Public component property. Becomes a kebab-cased attribute on the host. \
             Read as `NAME()`, written as `NAME.set(v)`. `!` marks required.",
            &["prop value: string = ''", "prop count: number! = 0"],
            &[("prop value = ''", "Type annotation required for typed reactivity.")]),
        dir("state", "state NAME [: TYPE] = INITIAL",
            "Local signal. Same `NAME()` / `NAME.set(v)` access as props.",
            &["state open = false", "state items: string[] = []"], &[]),
        dir("derived", "derived NAME = EXPR",
            "Memoised computed signal. Re-runs on any reactive read inside `EXPR`. \
             Auto-tracked.",
            &["derived total = items().length"], &[]),
        dir("data", "data NAME = ship.<domain>.<op>(ARGS)",
            "Server-sourced signal — implicitly live. Subscribed via SSE; initial \
             value is `null`. RHS must be a `ship.*` schema-source call. For local \
             seed, use `state`.",
            &["data agents = ship.agents.summaries()"],
            &[("data live X = ship.x.y()", "`live` keyword removed; live is implicit."),
              ("data X = 42", "RHS must be `ship.<domain>.<op>(...)`. Use `state` for seed.")]),
        dir("function", "function NAME(ARGS) { BODY }",
            "Plain JS in component scope. Has access to all injected scope bindings.",
            &["function close() { open.set(false) }"], &[]),
        dir("event", "event NAME: TYPE",
            "Typed CustomEvent declaration. Surfaces in the `.d.ts` as a known \
             dispatch target.",
            &["event submit: { value: string }"], &[]),
        dir("component_form_associated", "component formAssociated",
            "Element-level config. Attaches `ElementInternals` and exposes it as \
             `internals`. Required before any `on form_*` hook.",
            &["component formAssociated"], &[]),
    ]
}

fn scope_bindings() -> Vec<ScopeBinding> {
    let scope = vs(&["mount", "effect", "function", "form_lifecycle"]);
    let s = |n: &str, t: &str, d: &str| ScopeBinding {
        name: n.into(), ty: t.into(),
        available_in: scope.clone(), description: d.into(),
    };
    vec![
        s("el", "HTMLElement",
          "The host custom element instance. Use for `el.dispatchEvent`, \
           `el.querySelector`, `el.setAttribute`, etc."),
        s("internals", "ElementInternals | undefined",
          "Set when `component formAssociated` is declared. Use for \
           `internals.setFormValue(...)` and friends."),
        s("props", "Readonly<Record<string, unknown>>",
          "Read-only proxy. `props.X` is `X()` — alternate access shape."),
        s("cleanups", "Array<() => void>",
          "Push disposers for `mount`-time setup that needs teardown. The runtime \
           calls them on unmount."),
        s("slotFilled", "(host: HTMLElement, name: string) => boolean",
          "Returns true iff the named slot received caller-provided content."),
    ]
}

fn lifecycle_hooks() -> Vec<LifecycleHook> {
    let h = |n: &str, sy: &str, f: &str, d: &str| LifecycleHook {
        name: n.into(), syntax: sy.into(), fires: f.into(), description: d.into(),
    };
    vec![
        h("mount", "mount { BODY }",
          "Once on `connectedCallback` after the DOM is wired.",
          "One-time setup. Push to `cleanups` for subscriptions that need teardown."),
        h("effect", "effect { BODY }",
          "On any reactive read inside the body changing.",
          "Auto-tracked — do NOT use `effect on X { … }`."),
        h("form_reset", "on form_reset { BODY }",
          "When the parent `<form>` is reset (form-associated only).",
          "Lifecycle hook for `formResetCallback`. Requires `component formAssociated`."),
        h("form_restore", "on form_restore(state) { BODY }",
          "When the browser restores form state (BFCache / autofill).",
          "Lifecycle hook for `formStateRestoreCallback`. Param name pulls from \
           `e.detail.<param>`."),
        h("form_disabled", "on form_disabled(disabled) { BODY }",
          "When the host form's `disabled` state toggles.",
          "Lifecycle hook for `formDisabledCallback`."),
    ]
}

fn template_syntax() -> Vec<TemplateSyntax> {
    let t = |n: &str, sy: &str, d: &str, ex: &[&str]| TemplateSyntax {
        name: n.into(), syntax: sy.into(), description: d.into(), examples: vs(ex),
    };
    vec![
        t("interpolation", "{EXPR}",
          "Reactive text interpolation. Re-renders the text node when any signal \
           read inside `EXPR` changes.",
          &["<span>{count()}</span>"]),
        t("attribute_interpolation", "ATTR={EXPR}",
          "Reactive attribute. Empty / null / undefined / `false` auto-omit. \
           `true` becomes presence-only.",
          &["<div title={tooltip()} />"]),
        t("event_attribute", "ON_EVENT={HANDLER}",
          "Any `on*` attr with an interpolated value is an event listener. Use \
           platform names; `on:click` prefix is removed.",
          &["<button onclick={close}>close</button>"]),
        t("two_way_binding", "bind:PROP={SIGNAL}",
          "Two-way binding for form inputs. The runtime owns the input listener.",
          &["<input bind:value={value} />"]),
        t("if_block", "{#if COND}…{:else if COND}…{:else}…{/if}",
          "Mustache control flow. `:elif` is also accepted as shorthand.",
          &["{#if hasItems}<ul>…</ul>{:else}empty{/if}"]),
        t("for_block", "{#for ITEM of ITER}…[{:empty}…]{/for}",
          "List rendering. Optional `key={EXPR}` for stable identity.",
          &["{#for x of items() (key={x.id})}<li>{x.name}</li>{/for}"]),
        t("await_block", "{#await PROMISE}…{:then VALUE}…{:catch ERROR}…{/await}",
          "Async control flow. Loading branch until the promise settles.",
          &["{#await load()}…{:then v}{v}{:catch e}{e.message}{/await}"]),
        t("raw_html", "{@html EXPR}",
          "Raw HTML interpolation. Author owns sanitisation.",
          &["{@html sanitised}"]),
        t("slot", "<slot [name=\"NAME\"]>FALLBACK</slot>",
          "Light-DOM slot. Body is the fallback when no caller content arrives.",
          &["<slot name=\"icon\"><ship-icon name=\"info\" /></slot>"]),
        t("slot_fill", "slot=\"NAME\" on a child of a custom-element call",
          "Routes a child into the named slot of the consumed component. The \
           `slot=` attr is stripped after distribution; query via \
           `slotFilled(el, 'name')`.",
          &["<ship-tile><span slot=\"trailing\">→</span></ship-tile>"]),
        t("host_attrs", "<:host ATTR={EXPR} … />",
          "Reactive bindings on the component's host element. Replaces the \
           `mount + effect + el.setAttribute` ritual. Self-closing only.",
          &["<:host role={interactive() ? 'button' : 'listitem'} />"]),
        t("style_external", "<style src=\"./PATH.css\"></style>",
          "External stylesheet. Inlined at compile time.",
          &["<style src=\"./my-component.css\"></style>"]),
        t("component_call", "<TAG PROP={EXPR} ON_EVENT={FN}>CHILDREN</TAG>",
          "Custom-element invocation. Resolved against the registry.",
          &["<ship-list-tile variant=\"card\" onclick={open}>…</ship-list-tile>"]),
    ]
}

fn reactivity_rules() -> Vec<ReactivityRule> {
    let r = |id: &str, n: &str, d: &str, a: &str, f: &str| ReactivityRule {
        id: id.into(), name: n.into(), description: d.into(),
        anti_example: a.into(), fix_example: f.into(),
    };
    vec![
        r("R1", "signal_call_required",
          "Reading a signal in expression context requires `()`. Bare identifiers \
           reference the signal accessor itself.",
          "{count}", "{count()}"),
        r("R2", "no_signal_set_in_template",
          "Mutations belong in handlers, not templates.",
          "<button onclick={count.set(count() + 1)}>",
          "function inc() { count.set(count() + 1) }; <button onclick={inc}>"),
        r("R3", "no_effect_dependency_list",
          "`effect` auto-tracks reactive reads. Legacy `effect on X { … }` is removed.",
          "effect on count { console.log(count()) }",
          "effect { console.log(count()) }"),
        r("R4", "platform_event_attrs",
          "Event handlers use platform attribute names. `on:click` prefix is removed.",
          "<button on:click={fn} />", "<button onclick={fn} />"),
        r("R5", "no_bind_with_oninput",
          "`bind:value` already attaches an input listener. Don't pair with `oninput`.",
          "<input bind:value={v} oninput={fn} />",
          "<input bind:value={v} /> + effect { fn(v()) }"),
        r("R6", "data_is_implicitly_live",
          "`data` is server-sourced and always live. Static seeds use `state X = expr`.",
          "data live items = ship.items.list()",
          "data items = ship.items.list()"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_known_directives() {
        let m = emit();
        let names: Vec<&str> = m.directives.iter().map(|d| d.name.as_str()).collect();
        for required in ["prop", "state", "derived", "data", "function",
                         "event", "component_form_associated"] {
            assert!(names.contains(&required), "missing directive: {required}");
        }
        for hook in ["mount", "effect", "form_reset", "form_restore", "form_disabled"] {
            assert!(m.lifecycle_hooks.iter().any(|h| h.name == hook), "missing hook: {hook}");
        }
        for binding in ["el", "internals", "props", "cleanups", "slotFilled"] {
            assert!(m.scope_bindings.iter().any(|b| b.name == binding), "missing binding: {binding}");
        }
    }

    #[test]
    fn covers_every_node_variant() {
        // Every Node variant in parse::ast::Node must map to an entry.
        let m = emit();
        let names: Vec<&str> = m.template_syntax.iter().map(|t| t.name.as_str()).collect();
        for r in ["interpolation", "attribute_interpolation", "event_attribute",
                  "two_way_binding", "if_block", "for_block", "await_block",
                  "raw_html", "slot", "slot_fill", "host_attrs", "component_call"] {
            assert!(names.contains(&r), "missing template syntax: {r}");
        }
    }

    #[test]
    fn serialization_round_trips() {
        let m = emit();
        let json = serde_json::to_string(&m).expect("serialize");
        let back: Manifest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(m, back);
        assert_eq!(back.version, MANIFEST_VERSION);
    }

    #[test]
    fn reactivity_rules_well_formed() {
        let m = emit();
        assert!(!m.reactivity_rules.is_empty());
        for r in &m.reactivity_rules {
            assert!(r.id.starts_with('R'), "rule id should be R<n>: {}", r.id);
            assert!(!r.anti_example.is_empty());
            assert!(!r.fix_example.is_empty());
        }
    }
}
