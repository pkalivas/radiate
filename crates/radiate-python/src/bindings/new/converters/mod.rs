use radiate::Executor;

use crate::{PyEngineInput, PyEngineInputType};

mod alters;
mod selectors;

pub trait InputConverter<O> {
    fn convert(&self) -> O;
}

impl InputConverter<Executor> for Vec<PyEngineInput> {
    fn convert(&self) -> Executor {
        if self.len() != 1 {
            panic!("Expected exactly one executor, got {}", self.len());
        }

        let input = self
            .iter()
            .filter(|i| i.input_type == PyEngineInputType::Executor)
            .next();

        if let Some(input) = input {
            match input.component.as_str() {
                "Serial" => Executor::Serial,
                "FixedSizedWorkerPool" => {
                    let num_workers = input
                        .args()
                        .get("num_workers")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(1);

                    Executor::FixedSizedWorkerPool(num_workers)
                }
                "WorkerPool" => Executor::WorkerPool,
                _ => panic!("Executor type {} not yet implemented", input.component),
            }
        } else {
            Executor::Serial
        }
    }
}
