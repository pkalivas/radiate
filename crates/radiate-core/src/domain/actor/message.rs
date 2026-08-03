use std::sync::Arc;

/// Anything that can ride the bus. Blanket-implemented — the only real
/// requirement is being safe to hand across the `Executor`'s worker threads.
pub trait Message: Send + Sync + 'static {}
impl<M: Send + Sync + 'static> Message for M {}

/// A cheaply-clonable message envelope: wraps `D` in an `Arc` so fanning a
/// message out to many subscribed actors clones a pointer per actor, not the
/// payload itself. Most concrete message types on the bus should be a type
/// alias over this rather than hand-rolling their own `Arc` wrapper.
pub struct Envelope<D>(Arc<D>);

impl<D> Envelope<D> {
    pub fn new(data: D) -> Self {
        Envelope(Arc::new(data))
    }
}

impl<D> Clone for Envelope<D> {
    fn clone(&self) -> Self {
        Envelope(Arc::clone(&self.0))
    }
}

impl<D> std::ops::Deref for Envelope<D> {
    type Target = D;

    fn deref(&self) -> &D {
        &self.0
    }
}
