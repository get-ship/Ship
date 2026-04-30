//! Output buffer with a silent-walk depth counter.
//!
//! While `silent > 0`, `push_*` is a no-op. Phase E-prep needs this so
//! the walker can advance its key counter past un-rendered control-flow
//! branches without polluting the rendered HTML — keeps the registry
//! and walker key indices in lockstep.

pub(crate) struct Out {
    inner: String,
    silent: u32,
}

impl Out {
    pub(crate) fn new() -> Self {
        Self { inner: String::new(), silent: 0 }
    }

    pub(crate) fn push_str(&mut self, s: &str) {
        if self.silent == 0 {
            self.inner.push_str(s);
        }
    }

    pub(crate) fn push(&mut self, c: char) {
        if self.silent == 0 {
            self.inner.push(c);
        }
    }

    pub(crate) fn enter_silent(&mut self) {
        self.silent += 1;
    }

    pub(crate) fn exit_silent(&mut self) {
        debug_assert!(self.silent > 0, "exit_silent without matching enter");
        self.silent -= 1;
    }

    /// Force the silent depth to zero (visible) and return the prior
    /// depth so it can be handed back to `restore_silent`. Phase E-2
    /// fragment rendering uses this at the target boundary: the walker
    /// is silent everywhere except inside the target's body, where it
    /// flips back to visible to emit the fragment HTML.
    pub(crate) fn save_make_visible(&mut self) -> u32 {
        let prev = self.silent;
        self.silent = 0;
        prev
    }

    /// Restore a depth captured by `save_make_visible`.
    pub(crate) fn restore_silent(&mut self, prev: u32) {
        self.silent = prev;
    }

    pub(crate) fn into_string(self) -> String {
        self.inner
    }
}
