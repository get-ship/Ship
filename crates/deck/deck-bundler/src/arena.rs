//! Arena reclamation after a bundle (#256).
//!
//! Each `bundle_target` constructs a fresh rolldown/oxc bundler and loads the
//! target's full module graph into transient arenas. The graph is dropped when
//! the bundler goes out of scope, but **glibc's allocator does not return the
//! freed arenas to the OS** — repeated multi-hundred-MB bursts ratchet RSS up
//! and never recover, presenting as a monotonic leak. `malloc_trim(0)` forces
//! glibc to release that free top-of-heap back to the OS.
//!
//! This is the lower-risk of the two options in the #256 plan: a single FFI
//! call at the tail of each build, versus swapping the daemon's global
//! allocator to jemalloc (a process-wide `#[global_allocator]` change with
//! wider blast radius). It is a no-op on non-glibc targets — musl, macOS, and
//! Windows manage their heaps differently and don't strand freed arenas the
//! same way, and `malloc_trim` is a glibc extension that isn't present there.

/// Return freed allocator arenas to the OS. glibc-only; a no-op elsewhere.
///
/// Call after a `bundle_target` completes (success or error) so the transient
/// module-graph allocations don't accumulate as untrimmed glibc top-of-heap.
#[inline]
pub(crate) fn trim_arenas() {
    // `target_env = "gnu"` is glibc (where `malloc_trim` lives); musl and
    // every non-linux target skip it.
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    unsafe {
        // `malloc_trim(0)` trims all releasable top-of-heap. Best-effort:
        // the return value (1 = memory released, 0 = nothing to release)
        // is purely informational, so we ignore it.
        libc::malloc_trim(0);
    }
}
