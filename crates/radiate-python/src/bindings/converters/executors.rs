use crate::{
    PyEngineInput, PyEngineInputType,
    bindings::converters::{FromPyEngineInput, Registry},
};
use radiate::{Executor, RadiateResult};

fn executors() -> Registry<Executor> {
    use crate::constants::components;
    Registry {
        kind: "executor",
        entries: &[
            (components::SERIAL_EXECUTOR, serial),
            (components::WORKER_POOL_EXECUTOR, worker_pool),
            (
                components::FIXED_SIZED_WORKER_POOL_EXECUTOR,
                fixed_sized_worker_pool,
            ),
        ],
    }
}

impl FromPyEngineInput for Executor {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Executor)?;
        executors().convert(input)
    }
}

fn serial(_: &PyEngineInput) -> RadiateResult<Executor> {
    Ok(Executor::Serial)
}

fn worker_pool(_: &PyEngineInput) -> RadiateResult<Executor> {
    Ok(Executor::WorkerPool)
}

fn fixed_sized_worker_pool(input: &PyEngineInput) -> RadiateResult<Executor> {
    let num_workers = input.extract::<i64>("num_workers")?;
    Ok(Executor::FixedSizedWorkerPool(num_workers as usize))
}
