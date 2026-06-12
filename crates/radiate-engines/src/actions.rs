#[cfg(feature = "serde")]
use crate::FileWriter;
use crate::{EvolutionContext, Generation, runtime::RuntimeAction};
use radiate_core::{Chromosome, Engine, Objective, error::RadiateResult};
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "serde")]
use std::path::PathBuf;
use tracing::info;

pub(crate) struct LoggingAction(pub usize);

impl<C, T, E> RuntimeAction<E> for LoggingAction
where
    E: Engine<Epoch = Generation<C, T>, Ctx = EvolutionContext<C, T>>,
    C: Chromosome + Clone,
    T: Clone + Send + Sync,
{
    fn execute(&mut self, ctx: &E::Ctx) -> RadiateResult<()> {
        if !ctx.index.is_multiple_of(self.0) {
            return Ok(());
        }

        let time = ctx
            .metrics
            .time()
            .and_then(|m| m.times().map(|t| t.sum()))
            .unwrap_or_default();

        match ctx.objective {
            Objective::Single(_) => {
                info!(
                    "Epoch {:<4} | Score: {:>8.4} | Time: {:>5.2?}",
                    ctx.index,
                    ctx.score.as_ref().unwrap().as_f32(),
                    time
                );
            }
            Objective::Multi(_) => {
                let front_size = ctx.metrics.front_size();
                let front_size_value = front_size.map(|ent| ent.last_value()).unwrap_or(0.0);
                info!(
                    "Epoch {:<4} | Front Size: {:.3} | Time: {:>5.2?}",
                    ctx.index, front_size_value, time
                );
            }
        }

        Ok(())
    }
}

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
    fn execute(&mut self, guard: &E::Ctx) -> RadiateResult<()> {
        if guard.index.is_multiple_of(self.interval) {
            let file_path = self.path.join(format!(
                "chckpnt_{}.{}",
                guard.index,
                self.writer.extension()
            ));

            self.writer.write(file_path, &E::Epoch::from(guard))?;
        }

        Ok(())
    }
}
