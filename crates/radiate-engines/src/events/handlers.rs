use crate::events::events::*;

pub trait EventHandler<T>: Send + Sync {
    fn handle(&mut self, event: &EngineEvent<T>);
}

impl<T, F> EventHandler<T> for F
where
    F: Fn(&EngineEvent<T>) + Send + Sync,
{
    fn handle(&mut self, event: &EngineEvent<T>) {
        (self)(event)
    }
}
