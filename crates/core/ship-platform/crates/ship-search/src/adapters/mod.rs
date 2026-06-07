//! Per-kind search adapters. Each adapter consumes a request-time
//! snapshot of its source (a list of agent ids, a list of views with
//! manifest descriptions, a sessions registry handle, …) and applies
//! the shared scorer in `super::ranking`. They never reach across
//! kinds — `super::aggregator` is the only place that mixes results.

pub mod agents;
pub mod artifacts;
pub mod collection_items;
pub mod collections;
pub mod sessions;
pub mod skills;
pub mod views;
