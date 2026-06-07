//! Event validation — ingress checks before persistence and delivery.

use ship_types::envelope::EventEnvelope;

use crate::KernelError;

/// Validates an event before the kernel persists and routes it.
pub trait EventValidator: Send + Sync {
    fn validate(&self, event: &EventEnvelope) -> Result<(), KernelError>;
}

/// Blanket impl so closures work as validators.
impl<F> EventValidator for F
where
    F: Fn(&EventEnvelope) -> Result<(), KernelError> + Send + Sync,
{
    fn validate(&self, event: &EventEnvelope) -> Result<(), KernelError> {
        (self)(event)
    }
}
