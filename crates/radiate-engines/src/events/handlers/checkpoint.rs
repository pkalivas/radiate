#![cfg(feature = "serde")]

use crate::Generation;
use crate::{
    Actor,
    events::{ActorContext, GenerationSnapshot, MessageHandler},
};
use crate::{
    FileWriter,
    events::{CheckpointSaved, Warning},
};
use radiate_core::Chromosome;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct CheckpointWriterHandler<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    pub(crate) interval: usize,
    pub(crate) path: PathBuf,
    pub(crate) writer: Arc<Mutex<dyn FileWriter<Generation<C, T>> + Send + Sync>>,
}

impl<C, T> CheckpointWriterHandler<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    pub fn new<F>(interval: usize, path: PathBuf, writer: F) -> Self
    where
        F: FileWriter<Generation<C, T>> + Send + Sync + 'static,
    {
        Self {
            interval,
            path,
            writer: Arc::new(Mutex::new(writer)),
        }
    }
}

impl<C, T> Actor for CheckpointWriterHandler<C, T>
where
    C: Chromosome + Clone + Serialize + 'static,
    T: Clone + Send + Sync + Serialize + 'static,
{
    fn name(&self) -> &str {
        "CheckpointWriterHandler"
    }

    fn started(&mut self, ctx: &crate::events::ActorContext<Self>)
    where
        Self: Sized,
    {
        ctx.subscribe::<GenerationSnapshot<C, T>>()
            .schedule(self.interval);
    }
}

impl<C, T> MessageHandler<GenerationSnapshot<C, T>> for CheckpointWriterHandler<C, T>
where
    C: Chromosome + Clone + Serialize + 'static,
    T: Clone + Send + Sync + Serialize + 'static,
{
    fn handle(&mut self, message: &GenerationSnapshot<C, T>, ctx: &ActorContext<Self>) {
        let generation = &message.generation;
        let mut writer = self.writer.lock().unwrap();

        let file_path = self.path.join(format!(
            "chckpnt_{}.{}",
            generation.index(),
            writer.extension()
        ));

        match writer.write(file_path.clone(), generation) {
            Ok(_) => {
                ctx.publish(CheckpointSaved {
                    index: generation.index(),
                    path: file_path.into_string().unwrap_or_default(),
                });
            }
            Err(err) => {
                ctx.publish(Warning(err.to_string()));
            }
        };
    }
}
