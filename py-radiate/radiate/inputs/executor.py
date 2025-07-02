from typing import Any, Dict
from .component import ComponentBase


class Executor(ComponentBase):
    def __init__(self, component: str, args: Dict[str, Any] = {}):
        super().__init__(component=component, args=args)

    @staticmethod
    def Serial() -> "Executor":
        """
        Single-threaded executor.
        :return: An Executor instance configured for single-threaded execution.
        """
        return Executor(component="Serial")

    @staticmethod
    def WorkerPool() -> "Executor":
        """
        Worker pool executor.
        :return: An Executor instance configured for worker pool execution.
        """
        return Executor(component="WorkerPool")

    @staticmethod
    def FixedSizedWorkerPool(num_workers: int) -> "Executor":
        """
        Fixed-sized worker pool executor.
        :param num_workers: The number of worker threads in the pool.
        :return: An Executor instance configured for a fixed-sized worker pool.
        """
        return Executor(
            component="FixedSizedWorkerPool", args={"num_workers": num_workers}
        )
