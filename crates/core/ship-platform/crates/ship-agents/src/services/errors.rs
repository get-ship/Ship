//! Shared service-error type for the file-backed sibling services
//! (skills / agents / permissions / mcp / rules).
//!
//! Each service used to hand-roll a near-identical `pub enum Error` with a
//! manual `Display` impl and a manual `From<FileError>`. The shape is hoisted
//! here once, derived with `thiserror`, and parameterised over a zero-sized
//! [`ServiceEntity`] marker so every service keeps its exact Display text
//! (entity noun, `bad id` hint) and its exact `FileError` mapping. Services
//! alias it back as `pub type Error = ServiceError<TheirEntity>;` so all
//! existing tuple-variant construction and match sites compile unchanged.

use ship_files::files::{Error as FileError, Scope};
use std::marker::PhantomData;
use std::path::PathBuf;

/// Per-service flavor of [`ServiceError`]: the entity noun used in Display
/// text plus the (rare) per-service overrides of the `FileError` mapping.
pub trait ServiceEntity {
    /// Noun used in the `not found` / `already exists` Display text.
    const NOUN: &'static str;

    /// Suffix appended to the `bad id` Display text (e.g. the id grammar).
    const BAD_ID_HINT: &'static str = "";

    /// How `FileError::Json` maps in. Default: a parse failure.
    fn map_json(s: String) -> ServiceError<Self>
    where
        Self: Sized,
    {
        ServiceError::Parse(s)
    }

    /// How `FileError::Patch` maps in. Default: a patch failure.
    fn map_patch(s: String) -> ServiceError<Self>
    where
        Self: Sized,
    {
        ServiceError::Patch(s)
    }

    /// How `FileError::PathEscape` maps in. Default: a bad id.
    fn map_path_escape(p: PathBuf) -> ServiceError<Self>
    where
        Self: Sized,
    {
        ServiceError::BadId(p.display().to_string())
    }
}

/// The one shared service error. Variant set is the union across the five
/// sibling services; a service that never produces a variant simply never
/// constructs it (all consumer matches are non-exhaustive).
#[derive(Debug, thiserror::Error)]
pub enum ServiceError<E: ServiceEntity> {
    #[error("{noun} `{id}` not found", noun = E::NOUN, id = .0)]
    NotFound(String),
    #[error("{noun} `{id}` already exists", noun = E::NOUN, id = .0)]
    AlreadyExists(String),
    #[error("bad id `{id}`{hint}", id = .0, hint = E::BAD_ID_HINT)]
    BadId(String),
    #[error("bad path `{0}`")]
    BadPath(String),
    #[error("parse: {0}")]
    Parse(String),
    #[error("patch: {0}")]
    Patch(String),
    #[error("io: {0}")]
    Io(String),
    /// KV-vars failure (skills). Display is the underlying message verbatim
    /// so transport-layer error text stays byte-identical to the previous
    /// `anyhow::Error` pass-through.
    #[error("{0}")]
    Vars(String),
    #[error("scope {0:?} not configured")]
    ScopeUnavailable(Scope),
    /// Uninhabited — anchors the `E` marker parameter. Can never be
    /// constructed (`Infallible` has no values).
    #[doc(hidden)]
    #[error("unreachable")]
    _Marker(std::convert::Infallible, PhantomData<E>),
}

impl<E: ServiceEntity> From<FileError> for ServiceError<E> {
    fn from(e: FileError) -> Self {
        match e {
            FileError::NotFound(p) => Self::NotFound(p.display().to_string()),
            FileError::AlreadyExists(p) => Self::AlreadyExists(p.display().to_string()),
            FileError::ScopeUnavailable(s) => Self::ScopeUnavailable(s),
            FileError::Json(s) => E::map_json(s),
            FileError::Patch(s) => E::map_patch(s),
            FileError::Io(e) => Self::Io(e.to_string()),
            FileError::PathEscape(p) => E::map_path_escape(p),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    enum TestEntity {}
    impl ServiceEntity for TestEntity {
        const NOUN: &'static str = "widget";
        const BAD_ID_HINT: &'static str = " — hint";
    }

    type Error = ServiceError<TestEntity>;

    #[test]
    fn display_texts_carry_entity_noun_and_hint() {
        assert_eq!(
            Error::NotFound("x".into()).to_string(),
            "widget `x` not found"
        );
        assert_eq!(
            Error::AlreadyExists("x".into()).to_string(),
            "widget `x` already exists"
        );
        assert_eq!(Error::BadId("x".into()).to_string(), "bad id `x` — hint");
        assert_eq!(Error::BadPath("p".into()).to_string(), "bad path `p`");
        assert_eq!(Error::Parse("boom".into()).to_string(), "parse: boom");
        assert_eq!(Error::Patch("boom".into()).to_string(), "patch: boom");
        assert_eq!(Error::Io("boom".into()).to_string(), "io: boom");
        assert_eq!(Error::Vars("boom".into()).to_string(), "boom");
        assert_eq!(
            Error::ScopeUnavailable(Scope::Project).to_string(),
            "scope Project not configured"
        );
    }

    #[test]
    fn implements_std_error() {
        fn assert_err<T: std::error::Error>() {}
        assert_err::<Error>();
    }
}
