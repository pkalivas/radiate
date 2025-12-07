use std::sync::{Arc, mpsc};

pub struct CommandChannel<T> {
    sender: Arc<mpsc::Sender<T>>,
    receiver: mpsc::Receiver<T>,
}

impl<T> CommandChannel<T> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            sender: Arc::new(tx),
            receiver: rx,
        }
    }

    pub fn dispatcher(&self) -> Arc<mpsc::Sender<T>> {
        Arc::clone(&self.sender)
    }

    pub fn next(&self) -> Result<T, mpsc::RecvError> {
        self.receiver.recv()
    }
}
