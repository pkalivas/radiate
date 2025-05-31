use std::fmt::{Debug, Display};

use super::events::Event;

pub trait EventHandler<T>: Send + Sync {
    fn handle(&self, event: Event<T>);
}

impl<T, F> EventHandler<T> for F
where
    F: Fn(Event<T>) + Send + Sync + 'static,
{
    fn handle(&self, event: Event<T>) {
        (self)(event)
    }
}

pub struct EventLogger {
    full: bool,
}

impl EventLogger {
    pub fn new(full: bool) -> Self {
        EventLogger { full }
    }
}

impl<T: Debug + Display> EventHandler<T> for EventLogger {
    fn handle(&self, event: Event<T>) {
        if self.full {
            println!("Event: {:?}", event);
        } else {
            println!("Event: [{:?}]: {}", event.id(), event.data());
        }
    }
}

impl Default for EventLogger {
    fn default() -> Self {
        EventLogger::new(false)
    }
}
