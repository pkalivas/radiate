#[cfg(feature = "serde")]
use crate::FileWriter;
use crate::{context::RuntimeContext, runtime::iter::EngineGuard};
use radiate_core::{Engine, Objective};
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "serde")]
use std::path::PathBuf;
use tracing::info;

pub trait RuntimeAction<E: Engine> {
    fn execute<'a>(&mut self, guard: &EngineGuard<'a, E>);
}

pub(crate) struct LoggingAction(pub usize);

impl<E> RuntimeAction<E> for LoggingAction
where
    E: Engine,
    E::Context: RuntimeContext,
{
    fn execute<'a>(&mut self, guard: &EngineGuard<'a, E>) {
        let snapshot = guard.view();
        if !snapshot.index().is_multiple_of(self.0) {
            return;
        }

        match snapshot.objective() {
            Objective::Single(_) => {
                info!(
                    "Epoch {:<4} | Score: {:>8.4} | Time: {:>5.2?}",
                    snapshot.index(),
                    snapshot.score().unwrap().as_f32(),
                    snapshot.time()
                );
            }
            Objective::Multi(_) => {
                let front_size = snapshot.metrics().front_size();
                let front_size_value = front_size.map(|ent| ent.last_value()).unwrap_or(0.0);
                info!(
                    "Epoch {:<4} | Front Size: {:.3} | Time: {:>5.2?}",
                    snapshot.index(),
                    front_size_value,
                    snapshot.time()
                );
            }
        }
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

/// Implementation of `Iterator` for [CheckpointIterator].
///
/// Each call to `next()` retrieves the next generation, and if the generation
/// index matches the checkpoint interval, it serializes and saves the generation
/// to a JSON file in the specified directory. The filename format is
/// `generation_{index}.json`.
#[cfg(feature = "serde")]
impl<E> RuntimeAction<E> for CheckpointAction<E>
where
    E: Engine,
    E::Context: RuntimeContext,
    E::Epoch: Serialize,
{
    fn execute<'a>(&mut self, guard: &EngineGuard<'a, E>) {
        let snapshot = guard.view();
        if snapshot.index().is_multiple_of(self.interval) {
            let file_path = self.path.join(format!(
                "chckpnt_{}.{}",
                snapshot.index(),
                self.writer.extension()
            ));

            let write_result = self.writer.write(file_path, &guard.epoch());

            if let Err(e) = write_result {
                eprintln!("Failed to write checkpoint: {e}");
            }
        }
    }
}
