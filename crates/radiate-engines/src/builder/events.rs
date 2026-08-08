#[cfg(feature = "serde")]
use crate::{FileWriter, Generation};
use crate::{
    GeneticEngineBuilder,
    message::{Event, EventHandler},
};
use radiate_core::Chromosome;
#[cfg(feature = "serde")]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct CheckpointParams<C, T>
where
    C: Chromosome + 'static,
    T: Clone + 'static,
{
    pub interval: Option<usize>,
    pub path: Option<PathBuf>,
    #[cfg(feature = "serde")]
    pub writer: Option<Arc<dyn FileWriter<Generation<C, T>> + Send + Sync>>,
}

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    pub fn subscribe<M>(self, handler: impl EventHandler<M>) -> Self
    where
        M: Event,
    {
        self.params.event_stream.subscribe(handler);
        self
    }

    #[cfg(feature = "serde")]
    pub fn checkpoint(mut self, interval: usize, folder_path: impl AsRef<Path>) -> Self {
        self.params.checkpoint_params.interval = Some(interval);
        self.params.checkpoint_params.path = Some(folder_path.as_ref().to_path_buf());
        self
    }

    #[cfg(feature = "serde")]
    pub fn checkpoint_with(
        mut self,
        interval: usize,
        folder_path: impl AsRef<Path>,
        writer: Box<dyn FileWriter<Generation<C, T>> + Send + Sync>,
    ) -> Self {
        self.params.checkpoint_params.interval = Some(interval);
        self.params.checkpoint_params.path = Some(folder_path.as_ref().to_path_buf());
        self.params.checkpoint_params.writer = Some(writer.into());
        self
    }
}
