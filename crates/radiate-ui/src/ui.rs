use crate::app::{App, InputEvent};
use crate::gate::StepGate;
use color_eyre::{Result, eyre::Context};
use radiate_engines::{
    Chromosome, Engine, EngineIterator, Generation, GeneticEngine, error::RadiateResult,
    sync::ArcExt,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

const KEY_REPEAT_DELAY: Duration = Duration::from_millis(100);

pub struct UiRuntime<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    inner: GeneticEngine<C, T>,
    step_gate: StepGate,
    dispatcher: Arc<mpsc::Sender<InputEvent<C>>>,
    stop_flag: Arc<AtomicBool>,
    app_thread: Option<std::thread::JoinHandle<Result<()>>>,
    key_thread: Option<std::thread::JoinHandle<Result<()>>>,
}

impl<C, T> UiRuntime<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    pub fn new(inner: GeneticEngine<C, T>, render_interval: Duration) -> Self {
        let app = App::new(render_interval);

        let (dispatch_one, dispatch_two) = app.dispatcher().into_pair();
        let (stop_one, stop_two) = Arc::pair(AtomicBool::new(false));
        let gate = app.gate();

        let app_thread = std::thread::spawn(move || {
            let terminal = ratatui::init();
            app.run(terminal)?;
            ratatui::restore();
            Ok(())
        });

        let key_thread = std::thread::spawn(move || {
            while !stop_two.load(Ordering::Relaxed) {
                if crossterm::event::poll(KEY_REPEAT_DELAY)? {
                    let event = crossterm::event::read()?;
                    dispatch_two
                        .send(InputEvent::Crossterm(event))
                        .context("Failed to send Crossterm event")?;
                }
            }

            Ok(())
        });

        Self {
            inner,
            step_gate: gate,
            stop_flag: stop_one,
            dispatcher: dispatch_one,
            app_thread: Some(app_thread),
            key_thread: Some(key_thread),
        }
    }

    pub fn iter(self) -> impl Iterator<Item = Generation<C, T>> {
        EngineIterator::new(self)
    }
}

impl<C, T> Engine for UiRuntime<C, T>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync + 'static,
{
    type Epoch = Generation<C, T>;

    #[inline]
    fn next(&mut self) -> RadiateResult<Self::Epoch> {
        self.step_gate.wait_for_permit(&self.stop_flag);

        let current = self.inner.next()?;

        let send_result = match current.index() {
            1 => self
                .dispatcher
                .send(InputEvent::EngineStart(current.objective().clone())),
            _ => self.dispatcher.send(InputEvent::EpochComplete(
                current.index(),
                current.metrics().clone(),
                current.score().clone(),
                current.front().cloned(),
            )),
        };

        if send_result.is_err() {
            // UI is gone. Stop driving the runtime.
            self.stop_flag.store(true, Ordering::SeqCst);
            // wake any blocked waits
            self.step_gate.set_paused(false);

            // Option A: return the epoch anyway (engine can continue headless)
            // Option B (recommended): return an error / stop signal to end iteration
            //
            // If RadiateResult has a “stop”/“terminated” variant, return that here.
        }

        Ok(current)
    }
}

impl<C, T> Drop for UiRuntime<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        if !self.stop_flag.load(Ordering::Relaxed) {
            self.dispatcher.send(InputEvent::EngineStop).unwrap();
        }

        self.step_gate.set_paused(false);

        if let Some(event_listener) = self.app_thread.take() {
            if let Err(e) = event_listener.join() {
                eprintln!("Error joining app thread: {:?}", e);
            }
        }

        self.stop_flag.store(true, Ordering::SeqCst);

        if let Some(key_listener) = self.key_thread.take() {
            if let Err(e) = key_listener.join() {
                eprintln!("Error joining key listener thread: {:?}", e);
            }
        }
    }
}
