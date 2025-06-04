use super::{EventHandler, events::Event};
use radiate_core::Executor;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct EventBus<T> {
    handlers: Vec<Arc<Mutex<dyn EventHandler<T>>>>,
    executor: Arc<Executor>,
}

impl<T> EventBus<T> {
    pub fn new(executor: Arc<Executor>, handlers: Vec<Arc<Mutex<dyn EventHandler<T>>>>) -> Self {
        EventBus { handlers, executor }
    }

    pub fn clear(&mut self) {
        self.handlers.clear();
    }

    pub fn register(&mut self, handler: Arc<Mutex<dyn EventHandler<T>>>) {
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
                clone_handler.lock().unwrap().handle(clone_event);
            });
        }
    }
}
