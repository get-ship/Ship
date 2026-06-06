//! `ship_ops!` — single-source declaration for a domain's read ops.
//!
//! One entry per op emits BOTH halves that used to be hand-synced:
//!   * the `OpDescriptor` (for `schema::OPS`, the `/ship/schema` endpoint, and
//!     SDK codegen), and
//!   * the dispatch arm (name → typed handler → `serde_json::Value`).
//!
//! The descriptor's `returns` is the handler's *actual* Rust return type via
//! `stringify!`, so `data.ts` can emit `DataHandle<ReturnType>` instead of
//! `DataHandle<unknown>`. This collapses the old OPS-vs-dispatch shadow
//! registry: a `ship.<domain>.<op>` is now defined in exactly one place.
//!
//! Each handler closure has the signature
//! `fn(&Value, &ApiState, &FilesService) -> Result<Ret, RegistryError>`
//! (mirrors the live `try_dispatch` params). Closures capture nothing, so
//! they coerce to fn pointers. Returns are named `Serialize` types — never an
//! anonymous `json!` — and `serde_json::to_value` is the only `Value` boundary.

/// Declare a domain's ops once. Generates:
///   * `pub const <SLICE>: &[OpDescriptor]` — folded into `schema::OPS`.
///   * `pub(super) fn try_dispatch(...)` — the domain's dispatch entry, called
///     by `registry::call`'s chain.
///
/// `sensitive:` is optional (defaults `false`), matching the descriptor field.
#[macro_export]
macro_rules! ship_ops {
    (
        $slice:ident;
        $(
            {
                name: $name:literal,
                desc: $desc:literal,
                perm: $perm:literal,
                args: $args:expr,
                ret: $ret:ty,
                $( sensitive: $sensitive:literal, )?
                run: $run:expr $(,)?
            }
        ),* $(,)?
    ) => {
        /// Op descriptors for this domain — generated from the same
        /// declarations as `try_dispatch`, then assembled into `schema::OPS`.
        pub const $slice: &[$crate::services::schema::OpDescriptor] = &[ $(
            $crate::services::schema::OpDescriptor {
                name: $name,
                description: $desc,
                args: $args,
                returns: ::std::stringify!($ret),
                permission: $perm,
                sensitive: $crate::ship_ops!(@sensitive $($sensitive)?),
            },
        )* ];

        /// Generated dispatch — each arm runs the typed handler and serializes
        /// its named return type; unknown names fall through to the next domain.
        pub(super) fn try_dispatch(
            name: &str,
            args: &::serde_json::Value,
            state: &$crate::ship_runtime::ApiState,
            files: &::ship_files::files::FilesService,
        ) -> ::std::result::Result<
            ::std::option::Option<::serde_json::Value>,
            $crate::services::registry::RegistryError,
        > {
            match name {
                $(
                    $name => {
                        let run: fn(
                            &::serde_json::Value,
                            &$crate::ship_runtime::ApiState,
                            &::ship_files::files::FilesService,
                        ) -> ::std::result::Result<
                            $ret,
                            $crate::services::registry::RegistryError,
                        > = $run;
                        let typed = run(args, state, files)?;
                        let value = ::serde_json::to_value(&typed).map_err(|e| {
                            $crate::services::registry::RegistryError::service(e.to_string())
                        })?;
                        ::std::result::Result::Ok(::std::option::Option::Some(value))
                    }
                )*
                _ => ::std::result::Result::Ok(::std::option::Option::None),
            }
        }
    };

    (@sensitive) => { false };
    (@sensitive $s:literal) => { $s };
}
