mod channel;
mod control;
mod group;
mod inbox;
mod scheduler;
mod thread_pool;

use std::sync::Arc;

pub use channel::{CommandChannel, IntoPair};
pub use control::ThreadSync;
pub use group::{WaitGroup, WaitGuard};
pub use inbox::Inbox;
pub use scheduler::{Next, Scheduler, Timer};
pub use thread_pool::{ThreadPool, WorkResult, get_thread_pool};

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
