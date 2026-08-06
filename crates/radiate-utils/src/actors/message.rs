use crate::{Actor, Addr, events::Warning};
use crate::{MessageHandler, actors::ProcessId};
use std::sync::{Arc, Mutex};

const DEAD_LETTER_QUEUE_SIZE: usize = 100;

#[derive(Clone, Debug)]
pub struct DeadLetter {
    pub message_type: &'static str,
    pub pid: ProcessId,
}

pub struct DeadLetterActor {
    max_size: usize,
    queue: Arc<Mutex<Vec<DeadLetter>>>,
}

impl DeadLetterActor {
    pub fn new(max_size: usize) -> Self {
        DeadLetterActor {
            max_size,
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Actor for DeadLetterActor {
    fn on_init(&mut self, addr: &Addr<Self>) {
        addr.subscribe::<DeadLetter>();
    }
}

impl MessageHandler<DeadLetter> for DeadLetterActor {
    fn handle(&mut self, message: DeadLetter, addr: &Addr<Self>) {
        let mut queue = self.queue.lock().unwrap();
        if queue.len() < self.max_size {
            queue.push(message);
        } else {
            // If the queue is full, we can choose to drop the message or handle it differently.
            // For now, we'll just drop it.
            addr.publish(Warning {
                index: 0, // You might want to set this to a meaningful value
                message: format!(
                    "Dead letter received for actor {:?}: message type {}",
                    message.pid, message.message_type
                ),
            });
        }
    }
}

impl Default for DeadLetterActor {
    fn default() -> Self {
        Self::new(DEAD_LETTER_QUEUE_SIZE)
    }
}

#[derive(Clone, Debug)]
pub struct ActorSubscribed {
    pub message_type: &'static str,
    pub pid: ProcessId,
    pub subscriber_count: usize,
}

#[derive(Clone, Debug)]
pub struct ActorPanicked {
    pub pid: ProcessId,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct ActorStopped {
    pub pid: ProcessId,
}

#[derive(Clone, Debug)]
pub struct ActorStarted {
    pub pid: ProcessId,
}

#[derive(Clone, Debug)]
pub struct ActorRegistered {
    pub pid: ProcessId,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Start;
