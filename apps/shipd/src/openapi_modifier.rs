//! Post-generation spec surgery for the Ship daemon OpenAPI.
//!
//! Ships two concerns that can't be expressed purely via `#[utoipa::path]`
//! attributes:
//!
//! 1. Server URL + `bearer_token` security scheme registration, plus a
//!    **global** default `security: [{ bearer_token: [] }]` at the spec root.
//! 2. Per-operation `security: []` overrides for the paths listed in
//!    `openapi_types::PUBLIC_PATHS` — health, auth pairing, webhook ingress,
//!    static view assets.
//!
//! Handlers that already declare `security()` in their `#[utoipa::path]`
//! macro still win; this pass simply reinforces the public override table
//! as the source of truth for reviewers auditing "is this endpoint really
//! unauthenticated?".

use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityRequirement, SecurityScheme};
use utoipa::openapi::ServerBuilder;
use utoipa::Modify;

use crate::openapi_types::PUBLIC_PATHS;

pub struct SecurityAndServer;

impl Modify for SecurityAndServer {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("opaque")
                        .description(Some(
                            "Daemon bearer token — found in ~/.ship/daemon.token after `ship daemon start`. \
                             Also accepted: session tokens minted by /auth/pair/confirm or /auth/server-attest.",
                        ))
                        .build(),
                ),
            );
        }

        openapi.servers = Some(vec![
            ServerBuilder::new()
                .url("http://localhost:9315")
                .description(Some("Local shipd daemon"))
                .build(),
        ]);

        // Global default: every operation requires a bearer token unless
        // overridden below.
        openapi.security = Some(vec![SecurityRequirement::new::<&str, [&str; 0], _>(
            "bearer_token",
            [],
        )]);

        // Per-operation overrides: mark specific (method, path) pairs as public.
        for (method, path) in PUBLIC_PATHS {
            if let Some(item) = openapi.paths.paths.get_mut(*path) {
                if let Some(op) = operation_for_method(item, method) {
                    op.security = Some(Vec::new());
                }
            }
        }
    }
}

/// Return a mutable reference to the named operation on a path-item, matching
/// the case-insensitive HTTP method string.
fn operation_for_method<'a>(
    item: &'a mut utoipa::openapi::path::PathItem,
    method: &str,
) -> Option<&'a mut utoipa::openapi::path::Operation> {
    match method.to_ascii_lowercase().as_str() {
        "get" => item.get.as_mut(),
        "post" => item.post.as_mut(),
        "put" => item.put.as_mut(),
        "delete" => item.delete.as_mut(),
        "patch" => item.patch.as_mut(),
        "head" => item.head.as_mut(),
        "options" => item.options.as_mut(),
        "trace" => item.trace.as_mut(),
        _ => None,
    }
}
