from .._rd import components
from .input import EngineInput, EngineInputType


class Executor(EngineInput):
    def __init__(self, component: str, **kwargs):
        super().__init__(
            component=component, input_type=EngineInputType.Executor, **kwargs
        )

    @staticmethod
    def Serial() -> "Executor":
        """
        Single-threaded executor.
        :return: An Executor instance configured for single-threaded execution.
        """
        return Executor(component=components.SERIAL_EXECUTOR)

    @staticmethod
    def WorkerPool() -> "Executor":
        """
        Worker pool executor.
        :return: An Executor instance configured for worker pool execution.
        """
        return Executor(component=components.WORKER_POOL_EXECUTOR)

    @staticmethod
    def FixedSizedWorkerPool(num_workers: int = 1) -> "Executor":
        """
        Fixed-sized worker pool executor.
        :param num_workers: The number of worker threads in the pool.
        :return: An Executor instance configured for a fixed-sized worker pool.
        """
        return Executor(
            component=components.FIXED_SIZED_WORKER_POOL_EXECUTOR,
            num_workers=num_workers,
        )
