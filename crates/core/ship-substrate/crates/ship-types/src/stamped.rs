//! Type-state wrapper: an [`EventEnvelope`] that has passed the stamp chokepoint.
//!
//! Production write paths (`SqliteEventStore::append`, `EventPersist::persist`)
//! accept only [`Stamped`]. The sole production constructor is private to this
//! crate and is invoked only by [`crate::stamp::apply_stamp`] (the chokepoint).
//! External crates cannot mint a `Stamped` without that function (or the
//! feature-gated test mint).

use crate::envelope::EventEnvelope;

/// Proof that an envelope has been stamped and validated for persistence.
///
/// The inner field is private. There is **no** public constructor: production
/// code obtains a `Stamped` only from [`crate::stamp::apply_stamp`] (same crate,
/// private field init). Tests mint via [`Stamped::unchecked`] behind the
/// `test-support` feature — production builds never enable it.
#[derive(Debug, Clone)]
pub struct Stamped {
    pub(crate) inner: EventEnvelope,
}

impl Stamped {
    /// Unchecked mint for tests. Production builds never enable `test-support`.
    #[cfg(feature = "test-support")]
    pub fn unchecked(inner: EventEnvelope) -> Self {
        Self { inner }
    }

    /// Borrow the stamped envelope.
    pub fn as_envelope(&self) -> &EventEnvelope {
        &self.inner
    }

    /// Consume and return the inner envelope.
    pub fn into_envelope(self) -> EventEnvelope {
        self.inner
    }
}

impl std::ops::Deref for Stamped {
    type Target = EventEnvelope;

    fn deref(&self) -> &EventEnvelope {
        &self.inner
    }
}

impl AsRef<EventEnvelope> for Stamped {
    fn as_ref(&self) -> &EventEnvelope {
        &self.inner
    }
}
