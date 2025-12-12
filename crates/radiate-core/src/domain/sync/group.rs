use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Clone)]
pub struct WaitGroup {
    inner: Arc<Inner>,
    total_count: Arc<AtomicUsize>,
}

struct Inner {
    counter: AtomicUsize,
    lock: Mutex<()>,
    cvar: Condvar,
}

impl WaitGroup {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                counter: AtomicUsize::new(0),
                lock: Mutex::new(()),
                cvar: Condvar::new(),
            }),
            total_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get_count(&self) -> usize {
        self.total_count.load(Ordering::Acquire)
    }

    pub fn guard(&self) -> WaitGuard {
        self.inner.counter.fetch_add(1, Ordering::AcqRel);
        self.total_count.fetch_add(1, Ordering::AcqRel);

        WaitGuard { wg: self.clone() }
    }

    /// Waits until the counter reaches zero.
    pub fn wait(&self) -> usize {
        if self.inner.counter.load(Ordering::Acquire) == 0 {
            return 0;
        }

        let lock = self.inner.lock.lock().unwrap();
        let _unused = self
            .inner
            .cvar
            .wait_while(lock, |_| self.inner.counter.load(Ordering::Acquire) != 0);

        self.get_count()
    }
}

pub struct WaitGuard {
    wg: WaitGroup,
}

impl Drop for WaitGuard {
    fn drop(&mut self) {
        if self.wg.inner.counter.fetch_sub(1, Ordering::AcqRel) == 1 {
            let _guard = self.wg.inner.lock.lock().unwrap();
            self.wg.inner.cvar.notify_all();
        }
    }
}
