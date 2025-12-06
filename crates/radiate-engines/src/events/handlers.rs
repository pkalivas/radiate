use crate::events::events::*;
use std::sync::Arc;

pub trait EventHandler<T>: Send + Sync {
    fn handle(&mut self, event: Arc<EngineEvent<T>>);
}

impl<T, F> EventHandler<T> for F
where
    F: Fn(Arc<EngineEvent<T>>) + Send + Sync,
{
    fn handle(&mut self, event: Arc<EngineEvent<T>>) {
        (self)(event)
    }
}
