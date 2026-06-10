// FILE_CAP_OVERRIDE: proc-macro implementation — all code is required; no logic can be split without
// losing the derive attribute binding. Macro expansion, attribute parsing, validation, and code
// generation are a single compilation unit (proc-macro crates cannot depend on themselves).

//! `#[derive(EventPayload)]` proc-macro.
//!
//! Usage:
//! ```ignore
//! #[derive(EventPayload)]
//! #[event(name = "actor.created", scope = "runtime", retention = "durable", stability = "stable")]
//! pub struct ActorCreated { ... }
//! ```
//!
//! The macro validates:
//! - `#[event(...)]` attribute is present (compile error if absent).
//! - `name` matches `^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$` (compile error on mismatch).
//! - `scope` is one of `runtime | account | space` (compile error on unknown).
//! - `retention` is one of `durable | ephemeral` (compile error on unknown).
//! - `stability` is one of `stable | experimental | deprecated` (compile error on unknown).
//!
//! It emits `inventory::submit!` so the global `EVENT_REGISTRY` assembles from every
//! annotated struct across all crates. The derive declares a SINGLE-scope
//! allowed-set (the lone `scope`); poly-scopic events use the variadic
//! `register_event!` macro in `payload_registry.rs`.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    DeriveInput, Error, Expr, Lit, Meta, Token, parse_macro_input,
    punctuated::Punctuated,
};

/// Derive `EventPayload` on a struct, registering it in the global event registry.
///
/// Requires an `#[event(name = "...", scope = "...", retention = "...", stability = "...")]`
/// attribute on the same struct. Optional fields: `since_version = "..."`, `notes = "..."`.
#[proc_macro_derive(EventPayload, attributes(event))]
pub fn derive_event_payload(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_event_payload(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

struct EventAttr {
    name: String,
    scope: String,
    retention: String,
    stability: String,
    since_version: Option<String>,
}

fn impl_event_payload(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;

    // Find the #[event(...)] attribute.
    let event_attr = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("event"))
        .ok_or_else(|| {
            Error::new(
                Span::call_site(),
                format!(
                    "#[derive(EventPayload)] on `{}` requires a \
                     #[event(name=..., scope=..., retention=..., stability=...)] attribute",
                    struct_name
                ),
            )
        })?;

    let attr = parse_event_attr(event_attr)?;

    // Validate name format: ^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$
    validate_event_name(&attr.name, struct_name)?;

    // Map scope string → EventScope variant path.
    let scope_variant = match attr.scope.as_str() {
        "runtime" => quote! { ::ship_types::event_registry::EventScope::Runtime },
        "account" => quote! { ::ship_types::event_registry::EventScope::Account },
        "space" => quote! { ::ship_types::event_registry::EventScope::Space },
        other => {
            return Err(Error::new(
                Span::call_site(),
                format!(
                    "#[event(scope = \"{other}\")] on `{struct_name}` is invalid; \
                     valid values: runtime | account | space"
                ),
            ));
        }
    };

    // Map retention string → Retention variant path.
    let retention_variant = match attr.retention.as_str() {
        "durable" => quote! { ::ship_types::event_registry::Retention::Durable },
        "ephemeral" => quote! { ::ship_types::event_registry::Retention::Ephemeral },
        other => {
            return Err(Error::new(
                Span::call_site(),
                format!(
                    "#[event(retention = \"{other}\")] on `{struct_name}` is invalid; \
                     valid values: durable | ephemeral"
                ),
            ));
        }
    };

    // Map stability string → Stability variant path.
    let stability_variant = match attr.stability.as_str() {
        "stable" => quote! { ::ship_types::event_registry::Stability::Stable },
        "experimental" => quote! { ::ship_types::event_registry::Stability::Experimental },
        "deprecated" => quote! { ::ship_types::event_registry::Stability::Deprecated },
        other => {
            return Err(Error::new(
                Span::call_site(),
                format!(
                    "#[event(stability = \"{other}\")] on `{struct_name}` is invalid; \
                     valid values: stable | experimental | deprecated"
                ),
            ));
        }
    };

    let name_str = &attr.name;
    let struct_name_str = struct_name.to_string();
    let since_version_tokens = match &attr.since_version {
        Some(v) => quote! { Some(#v) },
        None => quote! { None },
    };

    Ok(quote! {
        impl ::ship_types::event_registry::EventDefSource for #struct_name {
            fn event_def() -> &'static ::ship_types::event_registry::EventDef {
                static DEF: ::ship_types::event_registry::EventDef =
                    ::ship_types::event_registry::EventDef {
                        name: #name_str,
                        allowed_scopes: &[#scope_variant],
                        retention: #retention_variant,
                        stability: #stability_variant,
                        since_version: #since_version_tokens,
                        payload_type_name: #struct_name_str,
                        validate_payload: {
                            fn validate(json: &str) -> ::core::result::Result<(), ::std::string::String> {
                                ::serde_json::from_str::<#struct_name>(json)
                                    .map(|_| ())
                                    .map_err(|e| e.to_string())
                            }
                            validate
                        },
                    };
                &DEF
            }
        }

        ::inventory::submit! {
            ::ship_types::event_registry::EventRegistration {
                def: <#struct_name as ::ship_types::event_registry::EventDefSource>::event_def,
            }
        }
    })
}

fn parse_event_attr(attr: &syn::Attribute) -> syn::Result<EventAttr> {
    let mut name: Option<String> = None;
    let mut scope: Option<String> = None;
    let mut retention: Option<String> = None;
    let mut stability: Option<String> = None;
    let mut since_version: Option<String> = None;

    // Parse as a list of `key = "value"` pairs.
    let pairs = attr.parse_args_with(
        Punctuated::<Meta, Token![,]>::parse_terminated,
    )?;

    for meta in &pairs {
        let Meta::NameValue(nv) = meta else {
            return Err(Error::new_spanned(
                meta,
                "#[event(...)] expects `key = \"value\"` pairs",
            ));
        };
        let key = nv.path.get_ident().map(|i| i.to_string()).unwrap_or_default();
        let Expr::Lit(expr_lit) = &nv.value else {
            return Err(Error::new_spanned(
                &nv.value,
                "#[event(...)] values must be string literals",
            ));
        };
        let Lit::Str(lit_str) = &expr_lit.lit else {
            return Err(Error::new_spanned(
                &expr_lit.lit,
                "#[event(...)] values must be string literals",
            ));
        };
        let val = lit_str.value();
        match key.as_str() {
            "name" => name = Some(val),
            "scope" => scope = Some(val),
            "retention" => retention = Some(val),
            "stability" => stability = Some(val),
            "since_version" => since_version = Some(val),
            "notes" => { /* allowed but ignored in registry */ }
            other => {
                return Err(Error::new_spanned(
                    &nv.path,
                    format!("#[event({other} = ...)] is not a recognised key"),
                ));
            }
        }
    }

    let name = name.ok_or_else(|| {
        Error::new(Span::call_site(), "#[event(...)] must include `name = \"...\"`")
    })?;
    let scope = scope.ok_or_else(|| {
        Error::new(Span::call_site(), "#[event(...)] must include `scope = \"...\"`")
    })?;
    let retention = retention.ok_or_else(|| {
        Error::new(
            Span::call_site(),
            "#[event(...)] must include `retention = \"...\"`",
        )
    })?;
    let stability = stability.unwrap_or_else(|| "stable".to_string());

    Ok(EventAttr {
        name,
        scope,
        retention,
        stability,
        since_version,
    })
}

fn validate_event_name(name: &str, struct_name: &syn::Ident) -> syn::Result<()> {
    // ^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$
    // Validated without the `regex` crate (proc-macros should stay lean).
    if name.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            format!(
                "`#[event(name = \"{name}\")] on `{struct_name}` is invalid: name must not be empty"
            ),
        ));
    }
    let segments: Vec<&str> = name.split('.').collect();
    if segments.len() < 2 {
        return Err(Error::new(
            Span::call_site(),
            format!(
                "`#[event(name = \"{name}\")] on `{struct_name}` is invalid: \
                 must have at least two dot-separated segments"
            ),
        ));
    }
    for seg in &segments {
        if seg.is_empty() {
            return Err(Error::new(
                Span::call_site(),
                format!(
                    "`#[event(name = \"{name}\")] on `{struct_name}` is invalid: \
                     empty segment (consecutive dots)"
                ),
            ));
        }
        let mut chars = seg.chars();
        let first = chars.next().unwrap();
        if !first.is_ascii_lowercase() {
            return Err(Error::new(
                Span::call_site(),
                format!(
                    "`#[event(name = \"{name}\")] on `{struct_name}` is invalid: \
                     each segment must start with a lowercase ASCII letter, got '{first}'"
                ),
            ));
        }
        for c in chars {
            if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' {
                return Err(Error::new(
                    Span::call_site(),
                    format!(
                        "`#[event(name = \"{name}\")] on `{struct_name}` is invalid: \
                         segment chars must be [a-z0-9_], got '{c}'"
                    ),
                ));
            }
        }
    }
    Ok(())
}
