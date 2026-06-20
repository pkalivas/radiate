use crate::app::{App, GenerationEvent, InputEvent};
use color_eyre::{Result, eyre::Context};
use radiate_engines::{
    Chromosome, Engine, Generation, GeneticEngine, error::RadiateResult, sync::ArcExt,
};
use radiate_engines::{EngineControl, EngineRuntime, EvolutionContext};
use std::{
    sync::{Arc, atomic::Ordering, mpsc},
    time::Duration,
};

const KEY_REPEAT_DELAY: Duration = Duration::from_millis(100);

pub struct TuiEngine<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    inner: GeneticEngine<C, T>,
    control: EngineControl,
    dispatcher: Arc<mpsc::Sender<InputEvent<C>>>,
    app_thread: Option<std::thread::JoinHandle<Result<()>>>,
    key_thread: Option<std::thread::JoinHandle<Result<()>>>,
}

impl<C, T> TuiEngine<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    pub fn new(mut inner: GeneticEngine<C, T>, render_interval: Duration) -> Self {
        let control = inner.control();
        let app = App::new(render_interval, control.clone());

        let (dispatch_one, dispatch_two) = app.dispatcher().into_pair();
        let stop_flag = control.stop_flag();

        let app_thread = std::thread::spawn(move || {
            let terminal = ratatui::init();
            app.run(terminal)?;
            ratatui::restore();
            Ok(())
        });

        let key_thread = std::thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
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
            control,
            dispatcher: dispatch_one,
            app_thread: Some(app_thread),
            key_thread: Some(key_thread),
        }
    }

    pub fn iter(self) -> EngineRuntime<Self> {
        let control = self.control.clone();
        EngineRuntime::new(self, Some(control))
    }
}

impl<C, T> Engine for TuiEngine<C, T>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync + 'static,
{
    type Ctx = EvolutionContext<C, T>;
    type Epoch = Generation<C, T>;

    fn context(&self) -> &Self::Ctx {
        self.inner.context()
    }

    fn epoch(&self) -> Self::Epoch {
        self.inner.epoch()
    }

    #[inline]
    fn step(&mut self) -> RadiateResult<()> {
        self.inner.step()?;
        let current = self.inner.context();

        if self.control.is_stopped() {
            return Ok(());
        }

        if current.index() == 1 {
            self.dispatcher
                .send(InputEvent::EngineStart(Arc::clone(&current.front())))
                .unwrap();
        }

        let event = GenerationEvent::from(current);
        self.dispatcher
            .send(InputEvent::EpochComplete(event))
            .unwrap();

        Ok(())
    }
}

impl<C, T> Drop for TuiEngine<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        if !self.control.is_stopped() {
            self.dispatcher.send(InputEvent::EngineStop).unwrap();
        }

        self.control.set_paused(false);

        if let Some(event_listener) = self.app_thread.take()
            && let Err(e) = event_listener.join()
        {
            eprintln!("Error joining app thread: {:?}", e);
        }

        self.control.stop();

        if let Some(key_listener) = self.key_thread.take()
            && let Err(e) = key_listener.join()
        {
            eprintln!("Error joining key listener thread: {:?}", e);
        }
    }
}
