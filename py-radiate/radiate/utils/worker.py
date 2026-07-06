import threading


class BackgroundWorker:
    def __init__(self, target_func, *args, **kwargs):
        self.target = target_func
        self.args = args
        self.kwargs = kwargs
        self.thread = None

    def __enter__(self):
        self.thread = threading.Thread(
            target=self.target,
            args=self.args,
            kwargs=self.kwargs,
            daemon=True,
        )
        self.thread.start()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.thread and self.thread.is_alive():
            self.thread.join()
