//! Per-actor mailbox — the receive end of an event channel.

use ship_types::envelope::EventEnvelope;
use tokio::sync::mpsc;

/// The receive end of a per-actor message channel.
#[derive(Debug)]
pub struct Mailbox {
    rx: mpsc::Receiver<EventEnvelope>,
}

impl Mailbox {
    pub(crate) fn new(rx: mpsc::Receiver<EventEnvelope>) -> Self {
        Self { rx }
    }

    /// Construct a `Mailbox` from an external receiver.
    pub fn from_receiver(rx: mpsc::Receiver<EventEnvelope>) -> Self {
        Self { rx }
    }

    /// Receive the next event. Returns `None` if all senders have been dropped.
    pub async fn recv(&mut self) -> Option<EventEnvelope> {
        self.rx.recv().await
    }

    /// Try to receive without blocking.
    pub fn try_recv(&mut self) -> Result<EventEnvelope, mpsc::error::TryRecvError> {
        self.rx.try_recv()
    }
}
