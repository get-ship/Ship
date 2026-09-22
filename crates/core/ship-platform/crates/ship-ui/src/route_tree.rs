//! `route_tree` — the composed route-TREE law (Phase-4 J3).
//!
//! One pure fn: [`compose_route_tree`] takes the space's composed
//! `views: Vec<ViewDescriptor>` (already namespace-prefix-merged by
//! `stack_model::compose::compose_space`, so every view already carries its
//! owning extension's prefix) and emits ONE Rust-resident [`RouteTree`] —
//! pure domain data derived only from [`ViewDescriptor`]'s own fields. No
//! IO, no runtime deps.
//!
//! ## The law
//!
//! - Each view's full path is `prefix` + `route` (e.g. `ext-alpha` +
//!   `/dashboard` → `ext-alpha/dashboard`), split on `/` and nested into the
//!   tree one node per segment. The tree's root has no segment of its own —
//!   it is the composition seam, not a mounted view.
//! - A view mounts at the node its full path resolves to. Ancestor and
//!   descendant nodes on the same path can each carry their own view
//!   independently (e.g. `/settings` and `/settings/profile` both mount) —
//!   a child does not require an index view at every ancestor.
//! - Children are sorted by segment for byte-stable output. The composed
//!   view set carries no other order worth preserving.
//!
//! ## Seam: consumption is a separate concern
//!
//! This tree makes no claim about how a segment should be interpreted by
//! whatever renders it — dynamic params, catch-alls, layout/chrome
//! wrapping, and any other presentation concern are a consumer's job, not
//! this law's. **Deck adapter TBD — deck owns consumption.** Deck is
//! mid-transformation and is not treated as authoritative here; a consumer
//! reads `RouteNode.segment` / `RouteNode.view` and applies its own
//! semantics on top without this type needing to change shape.

use serde::{Deserialize, Serialize};

use crate::view_descriptor::ViewDescriptor;

/// One segment of the composed route tree. The root node (see
/// [`RouteTree`]) has an empty `segment` and no `view` — every mounted view
/// lives at a descendant.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteNode {
    /// This node's path segment (e.g. `"dashboard"`). Empty only for the
    /// tree root.
    pub segment: String,
    /// The view mounted at this exact path, if any.
    pub view: Option<ViewDescriptor>,
    /// Nested segments, sorted by `segment` for deterministic output.
    pub children: Vec<RouteNode>,
}

/// The composed route tree: one Rust-resident tree a future consumer reads
/// as input, keyed by the same `(prefix, route)` structure
/// `stack_model::compose::compose_space` already composed.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteTree {
    pub root: RouteNode,
}

/// Apply the route-tree composition law. Pure; deterministic for identical
/// inputs — the same `views` in any declaration order produce byte-identical
/// output (children are sorted by segment, not left in insertion order).
pub fn compose_route_tree(views: &[ViewDescriptor]) -> RouteTree {
    let mut root = RouteNode::default();
    for view in views {
        insert(&mut root, &path_segments(view), view);
    }
    sort_children(&mut root);
    RouteTree { root }
}

/// `prefix` + `route`, split into non-empty segments. A `route` of `/`
/// (the prefix's own index) contributes no additional segment — the view
/// mounts on the prefix's node directly.
fn path_segments(view: &ViewDescriptor) -> Vec<String> {
    format!("{}/{}", view.prefix, view.route)
        .split('/')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn insert(node: &mut RouteNode, segments: &[String], view: &ViewDescriptor) {
    let Some((head, rest)) = segments.split_first() else {
        node.view = Some(view.clone());
        return;
    };
    let child_pos = match node.children.iter().position(|c| &c.segment == head) {
        Some(pos) => pos,
        None => {
            node.children.push(RouteNode {
                segment: head.clone(),
                view: None,
                children: Vec::new(),
            });
            node.children.len() - 1
        }
    };
    insert(&mut node.children[child_pos], rest, view);
}

fn sort_children(node: &mut RouteNode) {
    node.children.sort_by(|a, b| a.segment.cmp(&b.segment));
    for child in &mut node.children {
        sort_children(child);
    }
}
