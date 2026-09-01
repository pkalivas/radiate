use crate::{
    Actor, EpochComplete, JsonWriter,
    events::{GenerationSnapshot, MessageHandler},
};
use crate::{EvolutionContext, Generation, runtime::RuntimeAction};
#[cfg(feature = "serde")]
use crate::{FileWriter, events::CheckpointSaved};
use radiate_core::{Chromosome, Engine, error::RadiateResult};
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "serde")]
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::AtomicUsize};

#[cfg(feature = "serde")]
pub struct CheckpointActor<C, T>
where
    C: Chromosome + Clone + Serialize + 'static,
    T: Clone + Send + Sync + Serialize + 'static,
{
    pub(crate) interval: usize,
    pub(crate) path: PathBuf,
    pub(crate) writer: Arc<Mutex<dyn FileWriter<Generation<C, T>> + Send + Sync>>,
}

#[cfg(feature = "serde")]
impl<C, T> CheckpointActor<C, T>
where
    C: Chromosome + Clone + Serialize + 'static,
    T: Clone + Send + Sync + Serialize + 'static,
{
    pub fn new(interval: usize) -> Self {
        Self {
            interval,
            path: PathBuf::new(),
            writer: Arc::new(Mutex::new(JsonWriter)),
        }
    }
}

impl<C, T> Actor for CheckpointActor<C, T>
where
    C: Chromosome + Clone + Serialize + 'static,
    T: Clone + Send + Sync + Serialize + 'static,
{
    fn name(&self) -> &str {
        "CheckpointActor"
    }

    fn started(&mut self, ctx: &crate::events::ActorContext<Self>)
    where
        Self: Sized,
    {
        ctx.subscribe::<GenerationSnapshot<C, T>>()
            .schedule(self.interval);
    }
}

impl<C, T> MessageHandler<GenerationSnapshot<C, T>> for CheckpointActor<C, T>
where
    C: Chromosome + Clone + Serialize + 'static,
    T: Clone + Send + Sync + Serialize + 'static,
{
    fn handle(
        &mut self,
        message: &GenerationSnapshot<C, T>,
        ctx: &crate::events::ActorContext<Self>,
    ) {
        println!(
            "Received EpochComplete message: {:?}",
            message.generation.index()
        );
    }
}
