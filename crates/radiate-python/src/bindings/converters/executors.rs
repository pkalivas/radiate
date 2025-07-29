use crate::{InputTransform, PyEngineInput, PyEngineInputType};
use radiate::Executor;

impl InputTransform<Option<Executor>> for PyEngineInput {
    fn transform(&self) -> Option<Executor> {
        if self.input_type != PyEngineInputType::Executor {
            return None;
        }

        Some(match self.component.as_str() {
            "Serial" => Executor::Serial,
            "FixedSizedWorkerPool" => {
                let num_workers = self.get_usize("num_workers").unwrap_or(1);

                Executor::FixedSizedWorkerPool(num_workers)
            }
            "WorkerPool" => Executor::WorkerPool,
            _ => panic!("Executor type {} not yet implemented", self.component),
        })
    }
}
