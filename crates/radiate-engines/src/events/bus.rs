use super::{EventHandler, events::Event};
use radiate_core::{Executor, WorkerPoolExecutor};
use std::sync::Arc;

#[derive(Clone)]
pub struct EventBus<T> {
    handlers: Vec<Arc<dyn EventHandler<T>>>,
    executor: Arc<WorkerPoolExecutor>,
}

impl<T> EventBus<T> {
    pub fn new(handlers: Vec<Arc<dyn EventHandler<T>>>) -> Self {
        EventBus {
            handlers,
            executor: Arc::new(WorkerPoolExecutor::default()),
        }
    }

    pub fn clear(&mut self) {
        self.handlers.clear();
    }

    pub fn register(&mut self, handler: Arc<dyn EventHandler<T>>) {
        self.handlers.push(handler);
    }

    pub fn emit(&self, event: T)
    where
        T: Send + Sync + 'static,
    {
        if self.handlers.is_empty() {
            return;
        }

        let wrapped_event = Event::new(event);
        for handler in self.handlers.iter() {
            let clone_handler = Arc::clone(handler);
            let clone_event = wrapped_event.clone();
            self.executor.submit(move || {
                clone_handler.handle(clone_event);
            });
        }
    }
}

impl<T> Default for EventBus<T>
where
    T: Send + Sync + 'static,
{
    fn default() -> Self {
        EventBus {
            handlers: Vec::new(),
            executor: Arc::new(WorkerPoolExecutor::new(4)),
        }
    }
}
