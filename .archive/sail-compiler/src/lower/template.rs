//! Lower the parsed template AST into emit-ready IR.
//!
//! Each `Node` becomes either:
//! - Static HTML to bake into SSR output, or
//! - A `RuntimeBinding` in the bindings list (anchor + kind + deps + evaluator)
//!
//! Anchors are stable IDs (`data-bid="b1"`) the emitted runtime resolves
//! via `querySelector` at hydrate time.
