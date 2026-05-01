//! IR walker. Owns the output buffer + key allocator + scope and emits
//! HTML one instruction at a time. Phase D-3 fills in control-flow
//! bodies against the initial scope; D-2 brought renderable-subset
//! evaluation into text/attribute output, and D-1 nailed the marker
//! shapes. On evaluator failure we fall back to the marker-only shape
//! so client-side attachment still works.
//!
//! ## Phase E-prep — registry parity
//!
//! Key allocation is shared with `bindings::BindingRegistry`. The
//! registry pre-walks the IR (every branch, taken or not) and assigns
//! `b<N>` keys; the walker walks the IR again here, emitting HTML, and
//! its `b<N>` keys must match the registry index-for-index. To keep
//! the counter aligned across un-rendered branches, the walker walks
//! every branch — but suppresses HTML output for the un-taken ones via
//! the `Out` wrapper's silent mode.
//!
//! ## Phase E-2 — fragment rendering
//!
//! `FragmentState` flips the walker into "search-then-emit-then-skip"
//! mode for `render::render_fragment`. The walker walks the entire IR
//! (so the registry counter stays in lockstep) but emits HTML only for
//! the body of the boundary whose `b<N>` key matches the requested
//! target. The wrapper itself (`<sui-if>` / `<sui-for>` / `<sui-match>`
//! element) is suppressed because the runtime already has the
//! wrapper in place — it's swapping the wrapper's children, not the
//! wrapper itself. See `render::render_fragment` for the entry point.
//!
//! Element / attribute emission lives in `walker_attrs.rs` to keep
//! this file under the 300-line cap.

use crate::ir::Instr;
use crate::keys::{BindingRegistry, KeyAllocator};

use super::control_flow;
use super::escape::escape_text_value;
use super::eval::{Scope, eval, stringify};
use super::out::Out;

/// Fragment-rendering state for `render::render_fragment`. The full
/// `render::render` path uses `Off` and never touches it.
#[derive(Debug)]
pub(super) enum FragmentState {
    Off,
    /// Walking the IR silently, looking for the boundary whose key
    /// matches the carried target string.
    Searching(String),
    /// Inside the target body. The carried u32 is the silent depth that
    /// was active at entry; restored on exit.
    Rendering(u32),
    /// Target found and emitted. Walking continues silently to keep the
    /// registry counter aligned but no further HTML is produced.
    Done,
}

pub(crate) struct Walker<'r> {
    pub(super) out: Out,
    pub(super) keys: KeyAllocator,
    pub(super) scope: Scope,
    pub(super) registry: &'r BindingRegistry,
    /// Counter for parity with the registry. Increments each time the
    /// walker would have allocated a key. Used by `next_key` to assert
    /// the registry's `b<N>` matches.
    pub(super) reg_idx: usize,
    pub(super) frag: FragmentState,
    /// View name — used to build `/_sui/mutation/<view>?form=<key>` for
    /// `<form mutation>` elements. Empty string when not in SSR context.
    pub(super) view_name: String,
    /// Action/method attrs staged by `emit_attr_marker` for a mutation
    /// form and drained at the end of `emit_open_tag`.
    pub(super) pending_form_inject: Vec<(String, String)>,
}

impl<'r> Walker<'r> {
    pub(crate) fn new(scope: Scope, registry: &'r BindingRegistry) -> Self {
        Self::new_named(scope, registry, String::new())
    }

    pub(crate) fn new_named(
        scope: Scope,
        registry: &'r BindingRegistry,
        view_name: String,
    ) -> Self {
        Self {
            out: Out::new(),
            keys: KeyAllocator::default(),
            scope,
            registry,
            reg_idx: 0,
            frag: FragmentState::Off,
            view_name,
            pending_form_inject: Vec::new(),
        }
    }

    /// Construct a walker pre-loaded for `render_fragment`: starts in
    /// silent mode so the IR walk produces no HTML until the target
    /// boundary is hit, then control-flow renderers flip back to visible
    /// for the boundary's body and silent again afterward.
    pub(crate) fn new_fragment(
        scope: Scope,
        registry: &'r BindingRegistry,
        target: String,
    ) -> Self {
        let mut w = Self::new(scope, registry);
        w.out.enter_silent();
        w.frag = FragmentState::Searching(target);
        w
    }

    pub(crate) fn into_html(self) -> String {
        self.out.into_string()
    }

    /// Allocate the next `b<N>` key. Asserts (in debug builds) that
    /// the registry's entry at the matching index has the same key
    /// string — drift means the two traversals fell out of sync.
    pub(super) fn next_key(&mut self) -> String {
        let key = self.keys.alloc();
        debug_assert_eq!(
            self.registry.get(self.reg_idx).map(|b| b.key.as_str()),
            Some(key.as_str()),
            "walker/registry key drift at index {}",
            self.reg_idx
        );
        self.reg_idx += 1;
        key
    }

    /// Walk a block of instructions with HTML output suppressed.
    /// Key allocation continues so the registry-parity counter stays
    /// in lockstep with un-taken branches.
    pub(super) fn walk_block_silent(&mut self, instrs: &[Instr]) {
        self.out.enter_silent();
        self.walk_block(instrs);
        self.out.exit_silent();
    }

    pub(crate) fn walk_block(&mut self, instrs: &[Instr]) {
        for instr in instrs {
            self.walk_instr(instr);
        }
    }

    pub(super) fn walk_instr(&mut self, instr: &Instr) {
        match instr {
            Instr::StaticText { text, .. } => self.out.push_str(text),
            Instr::Comment { data, .. } => {
                self.out.push_str("<!--");
                self.out.push_str(data);
                self.out.push_str("-->");
            }
            Instr::DynamicText { expr } => {
                let key = self.next_key();
                let inner = eval(&expr.source, &self.scope)
                    .map(|v| escape_text_value(&stringify(&v)))
                    .unwrap_or_default();
                self.out.push_str(&format!("<!--sui:text:{key}-->{inner}<!--/sui-->"));
            }
            Instr::Element(el) => self.walk_element(el),
            Instr::If(if_instr) => control_flow::render_if(self, if_instr),
            Instr::For(for_instr) => control_flow::render_for(self, for_instr),
            Instr::Match(match_instr) => control_flow::render_match(self, match_instr),
            Instr::WhileLoading(wl) => control_flow::render_while_loading(self, wl),
            Instr::WhileError(we) => control_flow::render_while_error(self, we),
        }
    }

    /// If the walker is searching for `key`, switch into rendering mode
    /// (visible output) and return true; the caller then suppresses the
    /// boundary wrapper. Otherwise return false; normal wrapper emission
    /// applies.
    pub(super) fn enter_target_if_match(&mut self, key: &str) -> bool {
        let hit = matches!(&self.frag, FragmentState::Searching(t) if t == key);
        if hit {
            let prev = self.out.save_make_visible();
            self.frag = FragmentState::Rendering(prev);
        }
        hit
    }

    /// Pair of `enter_target_if_match` for the rendering branch. Restore
    /// silent depth and mark the target as consumed.
    pub(super) fn exit_target(&mut self) {
        if let FragmentState::Rendering(prev) = self.frag {
            self.out.restore_silent(prev);
            self.frag = FragmentState::Done;
        }
    }

    /// True once the target boundary has been encountered (whether its
    /// body was non-empty or empty). `render_fragment` reads this after
    /// the walk to detect "registry says key exists but walker never hit
    /// it" — a debug-only invariant violation.
    pub(super) fn target_found(&self) -> bool {
        matches!(
            self.frag,
            FragmentState::Done | FragmentState::Rendering(_)
        )
    }
}
