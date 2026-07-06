use crate::{InputTransform, PyEngineInput, PyEngineInputType};
use radiate::{Executor, RadiateResult};
use radiate_error::radiate_bail;

impl InputTransform<RadiateResult<Executor>> for PyEngineInput {
    fn transform(&self) -> RadiateResult<Executor> {
        if self.input_type != PyEngineInputType::Executor {
            radiate_bail!(Builder: "Expected Executor input, got {:?}", self.input_type);
        }

        Ok(match self.component.as_str() {
            crate::constants::SERIAL_EXECUTOR => Executor::Serial,
            crate::constants::FIXED_SIZED_WORKER_POOL_EXECUTOR => {
                let num_workers = self.extract::<i64>("num_workers")?;
                Executor::FixedSizedWorkerPool(num_workers as usize)
            }
            crate::constants::WORKER_POOL_EXECUTOR => Executor::WorkerPool,
            _ => radiate_bail!(Builder: "Unknown executor type: {}", self.component),
        })
    }
}
