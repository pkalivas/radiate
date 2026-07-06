import threading


class BackgroundWorker:
    def __init__(self, target_function, *args, **kwargs):
        """
        Accepts any function and its arguments to run in the background.
        """
        self.target = target_function
        self.args = args
        self.kwargs = kwargs
        self.thread = None

    def __enter__(self):
        # Create and start the thread immediately
        self.thread = threading.Thread(
            target=self.target,
            args=self.args,
            kwargs=self.kwargs,
            daemon=True,  # Ensures the thread dies if the main program crashes
        )
        self.thread.start()
        return self  # Control moves instantly to the code inside the 'with' block

    def __exit__(self, exc_type, exc_val, exc_tb):
        # Wait for the background task to finish before leaving the block
        if self.thread and self.thread.is_alive():
            print("\n[Front] Main tasks done. Waiting for background task to finish...")
            self.thread.join()
        print("[Front] Background worker context closed.")
