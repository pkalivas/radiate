// use crate::{Metric, MetricSet, MetricUpdate};
// use std::sync::{Arc, Mutex, mpsc};

// enum BatchCommand {
//     Update(Vec<Metric>),
//     Complete,
// }

// pub struct BatchMetricUpdater {
//     set: Arc<Mutex<MetricSet>>,
//     thread: Option<std::thread::JoinHandle<()>>,
//     sender: mpsc::Sender<BatchCommand>,
// }

// impl BatchMetricUpdater {
//     pub fn new() -> Self {
//         let (sender, receiver) = mpsc::channel::<BatchCommand>();

//         let receiver = Arc::new(Mutex::new(receiver));
//         let set = Arc::new(Mutex::new(MetricSet::new()));
//         let thread_set = Arc::clone(&set);
//         let thread = std::thread::spawn(move || {
//             let updates = receiver.lock().unwrap().recv().unwrap();

//             match updates {
//                 BatchCommand::Update(metrics) => {
//                     let mut set = thread_set.lock().unwrap();
//                     for metric in metrics {
//                         set.add_or_update(metric);
//                     }
//                 }
//                 BatchCommand::Complete => return,
//             };
//         });

//         BatchMetricUpdater {
//             set,
//             thread: Some(thread),
//             sender,
//         }
//     }

//     #[inline]
//     pub fn send(&self, update: Vec<Metric>) {
//         let _ = self.sender.send(BatchCommand::Update(update));
//     }

//     #[inline]
//     pub fn complete(self) -> MetricSet {
//         if let Some(thread) = self.thread {
//             let _ = self.sender.send(BatchCommand::Complete);
//             let _ = thread.join();
//         }

//         Arc::try_unwrap(self.set)
//             .ok()
//             .map(|mutex| mutex.into_inner().unwrap())
//             .unwrap_or_else(MetricSet::new)
//     }
// }
