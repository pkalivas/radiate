
from radiate.radiate import PyExecutor

class Executor:
    """
    Executor types for evaluating fitness functions.
    """

    def __init__(self, executor: PyExecutor):
        if not isinstance(executor, PyExecutor):
            raise TypeError("executor must be an instance of PyExecutor.")
        self.executor = executor

    @staticmethod
    def Serial() -> "Executor":
        """
        Single-threaded executor.
        :return: An Executor instance configured for single-threaded execution.
        """
        return Executor(PyExecutor.serial())
    
    @staticmethod
    def WorkerPool() -> "Executor":
        """
        Worker pool executor.
        :param thread_pool: A ThreadPool instance to manage worker threads.
        :return: An Executor instance configured for worker pool execution.
        """
        return Executor(PyExecutor.worker_pool())
    
    @staticmethod
    def FixedSizedWorkerPool(num_workers: int) -> "Executor":
        """
        Fixed-sized worker pool executor.
        :param num_workers: The number of worker threads in the pool.
        :return: An Executor instance configured for a fixed-sized worker pool.
        """
        return Executor(PyExecutor.fixed_sized_worker_pool(num_workers))
