use crossbeam::channel;

pub struct CommandChannel<T> {
    sender: channel::Sender<T>,
    receiver: channel::Receiver<T>,
}

impl<T> CommandChannel<T> {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam::channel::unbounded();
        Self {
            sender: tx,
            receiver: rx,
        }
    }

    pub fn dispatcher(&self) -> channel::Sender<T> {
        self.sender.clone()
    }

    pub fn next(&self) -> Result<T, channel::RecvError> {
        self.receiver.recv()
    }
}

impl<T> Default for CommandChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Iterator for CommandChannel<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.recv().ok()
    }
}

pub trait IntoPair<T> {
    fn into_pair(self) -> (T, T);
}

impl<T> IntoPair<channel::Sender<T>> for channel::Sender<T> {
    fn into_pair(self) -> (channel::Sender<T>, channel::Sender<T>) {
        (self.clone(), self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_channel() {
        let channel: CommandChannel<i32> = CommandChannel::new();
        let dispatcher = channel.dispatcher();
        dispatcher.send(42).unwrap();
        assert_eq!(channel.next().unwrap(), 42);
    }

    #[test]
    fn test_command_channel_iterator() {
        let channel: CommandChannel<i32> = CommandChannel::new();
        let dispatcher = channel.dispatcher();
        dispatcher.send(1).unwrap();
        dispatcher.send(2).unwrap();

        for (i, value) in channel.into_iter().enumerate() {
            assert_eq!(value, (i + 1) as i32);
            if i == 1 {
                break;
            }
        }
    }
}
