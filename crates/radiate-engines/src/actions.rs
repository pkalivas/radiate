use crate::{EvolutionContext, Generation, runtime::RuntimeAction};
#[cfg(feature = "serde")]
use crate::{FileWriter, events::CheckpointSaved};
use radiate_core::{Chromosome, Engine, error::RadiateResult};
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "serde")]
use std::path::PathBuf;

#[cfg(feature = "serde")]
pub(crate) struct CheckpointAction<E>
where
    E: Engine,
    E::Epoch: Serialize,
{
    pub(crate) interval: usize,
    pub(crate) path: PathBuf,
    pub(crate) writer: Box<dyn FileWriter<E::Epoch>>,
}

#[cfg(feature = "serde")]
impl<C, T, E> RuntimeAction<E> for CheckpointAction<E>
where
    E: Engine<Epoch = Generation<C, T>, Ctx = EvolutionContext<C, T>>,
    C: Chromosome + Clone,
    T: Clone + Send + Sync,
    E::Epoch: Serialize,
{
    fn execute(&mut self, ctx: &E::Ctx) -> RadiateResult<()> {
        if ctx.index.is_multiple_of(self.interval) {
            let file_path =
                self.path
                    .join(format!("chckpnt_{}.{}", ctx.index, self.writer.extension()));

            self.writer.write(file_path.clone(), &E::Epoch::from(ctx))?;
            ctx.events().publish(CheckpointSaved {
                index: ctx.index,
                path: file_path.into_string().unwrap_or_default(),
            });
        }

        Ok(())
    }
}
