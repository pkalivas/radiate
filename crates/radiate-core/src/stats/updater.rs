// use crate::{MetricSet, WaitGroup, WaitGuard, stats::set::MetricSetUpdate};
// use std::{
//     sync::{Arc, Mutex, mpsc},
//     thread::JoinHandle,
// };

// #[allow(dead_code)]
// enum BatchCommand {
//     Update((WaitGuard, MetricSetUpdate)),
//     Complete,
// }

// pub enum ProcessorInner {
//     SingleThreaded(MetricSet),
//     Background(BackgroundMetricProcessor),
// }

// pub struct MetricProcessor {
//     inner: ProcessorInner,
// }

// impl MetricProcessor {
//     pub fn new() -> Self {
//         MetricProcessor {
//             inner: ProcessorInner::SingleThreaded(MetricSet::new()),
//         }
//     }

//     pub fn into_background(self) -> Self {
//         match self.inner {
//             ProcessorInner::SingleThreaded(set) => MetricProcessor {
//                 inner: ProcessorInner::Background(BackgroundMetricProcessor::from(set)),
//             },
//             ProcessorInner::Background(_) => self,
//         }
//     }

//     #[inline]
//     pub fn update(&mut self, update: impl Into<MetricSetUpdate>) {
//         match &mut self.inner {
//             ProcessorInner::SingleThreaded(set) => {
//                 set.add_or_update(update.into());
//             }
//             ProcessorInner::Background(processor) => {
//                 processor.update(update);
//             }
//         }
//     }

//     #[inline]
//     pub fn flush_into(&mut self, target: &mut MetricSet) {
//         match &mut self.inner {
//             ProcessorInner::SingleThreaded(set) => {
//                 set.flush_all_into(target);
//                 set.clear();
//             }
//             ProcessorInner::Background(processor) => {
//                 processor.flush_into(target);
//             }
//         }
//     }
// }

// pub struct BackgroundMetricProcessor {
//     metrics: Arc<Mutex<MetricSet>>,
//     thread: Option<JoinHandle<()>>,
//     waiter: Arc<WaitGroup>,
//     sender: mpsc::Sender<BatchCommand>,
// }

// impl BackgroundMetricProcessor {
//     pub fn new() -> Self {
//         let (sender, receiver) = mpsc::channel::<BatchCommand>();

//         let guard = Arc::new(WaitGroup::new());
//         let receiver = Arc::new(Mutex::new(receiver));
//         let metrics = Arc::new(Mutex::new(MetricSet::new()));
//         let thread_metrics = Arc::clone(&metrics);

//         BackgroundMetricProcessor {
//             metrics,
//             sender,
//             waiter: Arc::clone(&guard),
//             thread: Some(std::thread::spawn(move || {
//                 loop {
//                     let updates = receiver.lock().unwrap().recv().unwrap();

//                     match updates {
//                         BatchCommand::Update((waiter, metrics)) => {
//                             let mut set = thread_metrics.lock().unwrap();
//                             set.add_or_update(metrics);
//                             drop(waiter);
//                         }
//                         BatchCommand::Complete => return,
//                     };
//                 }
//             })),
//         }
//     }

//     #[inline]
//     pub fn update(&mut self, update: impl Into<MetricSetUpdate>) {
//         self.sender
//             .send(BatchCommand::Update((self.waiter.guard(), update.into())))
//             .unwrap();
//     }

//     #[inline]
//     pub fn flush_into(&mut self, target: &mut MetricSet) {
//         self.waiter.wait();
//         let mut set = self.metrics.lock().unwrap();
//         set.flush_all_into(target);
//         set.clear();
//     }
// }

// impl Drop for BackgroundMetricProcessor {
//     fn drop(&mut self) {
//         if let Some(thread) = self.thread.take() {
//             self.waiter.wait();
//             self.sender.send(BatchCommand::Complete).unwrap();
//             thread.join().unwrap();
//         }
//     }
// }

// impl From<MetricSet> for BackgroundMetricProcessor {
//     fn from(set: MetricSet) -> Self {
//         let processor = BackgroundMetricProcessor::new();

//         {
//             let mut processor_set = processor.metrics.lock().unwrap();
//             *processor_set = set;
//         }

//         processor
//     }
// }
