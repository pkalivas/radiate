use super::events::Event;

pub trait EventHandler<T>: Send + Sync {
    fn handle(&mut self, event: Event<T>);
}

impl<T, F> EventHandler<T> for F
where
    F: Fn(Event<T>) + Send + Sync,
{
    fn handle(&mut self, event: Event<T>) {
        (self)(event)
    }
}
