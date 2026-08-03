mod bus;
mod channel;
mod control;
mod group;
mod thread_pool;

use std::sync::Arc;

pub use bus::{ActorContext, ActorSystem, Envelope, EventHandler, Message};
pub use channel::CommandChannel;
pub use control::ThreadSync;
pub use group::{WaitGroup, WaitGuard};
pub use thread_pool::{WorkResult, get_thread_pool};

pub trait ArcExt<T> {
    fn pair(value: T) -> (Arc<T>, Arc<T>) {
        let arc = Arc::new(value);
        (Arc::clone(&arc), arc)
    }

    fn into_pair(self) -> (Arc<T>, Arc<T>);
}

impl<T> ArcExt<T> for Arc<T> {
    fn into_pair(self) -> (Arc<T>, Arc<T>) {
        (Arc::clone(&self), self)
    }
}
