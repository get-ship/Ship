//! Collections — structured, queryable data for any domain.
//!
//! A collection has a schema (`.ship/schemas/<id>.jsonc`), contains items
//! stored in `.ship/docs.db`, and can be queried by any surface.

pub mod builtin;
pub mod migration;
pub mod query;
pub mod schema;
pub mod service;
pub mod store;

#[cfg(test)]
mod tests_integration;
