//! ship-ui — the unified UI authoring service (L1).
//!
//! One crate owns the full authoring surface: component discovery (Packs,
//! Read), SUI compiler analysis (Diagnostics, Hover, Dependencies), and view
//! authoring writes (PatchSession, AppendSession, SignalSession, EditMode,
//! Tool, Scaffold). Replaces the retired `ship-views` + `SuiService` pair.
//!
//! Owns view-slice schemas ([`ViewDescriptor`]) and the route-tree
//! composition law ([`compose_route_tree`]) — products own their noun
//! schemas; stack-model references `ViewDescriptor` and consumers call
//! `compose_route_tree` when they hydrate the view slice.
//!
//! Depends only on substrate ports and the collection/artifact services
//! — no reach back into ship-platform.

pub mod registry;
pub mod route_tree;
pub mod themes_contrast;
pub mod view_authoring;
pub mod view_descriptor;
pub mod view_manifest;
pub mod view_ports;
pub mod wire;

pub use route_tree::{RouteNode, RouteTree, compose_route_tree};
pub use view_descriptor::ViewDescriptor;
