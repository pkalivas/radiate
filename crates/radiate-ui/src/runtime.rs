use crate::{
    DEFAULT_RENDER_INTERVAL,
    app::{App, GenerationEvent, ImprovementEvent, InputEvent},
};
use color_eyre::{Result, eyre::Context};
use crossbeam::channel;
use radiate_engines::{
    Chromosome, EngineStart, EventContext, EventHandler, Expr, GenerationSnapshot, Handler,
    Improvement, StreamClosed, StreamEvent, StreamSnapshot, error::RadiateResult, sync::IntoPair,
};
use radiate_engines::{LogEvent, ThreadSync};
use std::{fmt::Debug, sync::atomic::Ordering, time::Duration};

const KEY_REPEAT_DELAY: Duration = Duration::from_millis(100);

pub struct Tui<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    control: ThreadSync,
    dispatcher: channel::Sender<InputEvent<C>>,
    app_thread: Option<std::thread::JoinHandle<Result<()>>>,
    key_thread: Option<std::thread::JoinHandle<Result<()>>>,
    _marker: std::marker::PhantomData<T>,
}

impl<C, T> Tui<C, T>
where
    C: Chromosome + Debug + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    pub fn new(control: ThreadSync) -> Self {
        radiate_engines::disable_logging();
        let app = App::new(DEFAULT_RENDER_INTERVAL, control.clone());
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
            control,
            dispatcher: dispatch_one,
            app_thread: Some(app_thread),
            key_thread: Some(key_thread),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C, T> EventHandler for Tui<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
        let mut subs = Vec::new();

        subs.push(ctx.subscribe::<EngineStart<C>>());
        subs.push(ctx.subscribe::<GenerationSnapshot<C, T>>());
        subs.push(ctx.subscribe::<Improvement<T>>());
        subs.push(ctx.subscribe::<LogEvent>());
        subs.push(
            ctx.subscribe::<StreamEvent<StreamSnapshot>>()
                .schedule(Expr::every(10))?,
        );
        subs.push(ctx.subscribe::<StreamEvent<StreamClosed>>());

        for subscription in subs {
            self.dispatcher
                .send(InputEvent::Subscription(subscription))
                .context("Failed to send subscription")
                .unwrap();
        }

        Ok(())
    }
}

impl<C, T> Handler<EngineStart<C>> for Tui<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn handle(&mut self, event: &EngineStart<C>, _: &EventContext<'_, Self>) {
        self.dispatcher
            .send(InputEvent::EngineStart(event.front.clone()))
            .context("Failed to send EngineStart event")
            .unwrap();
    }
}

impl<C, T> Handler<GenerationSnapshot<C, T>> for Tui<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn handle(&mut self, event: &GenerationSnapshot<C, T>, _: &EventContext<'_, Self>) {
        self.dispatcher
            .send(InputEvent::EpochComplete(GenerationEvent::from(event)))
            .context("Failed to send GenerationSnapshot event")
            .unwrap();
    }
}

impl<C, T> Handler<Improvement<T>> for Tui<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn handle(&mut self, event: &Improvement<T>, _: &EventContext<'_, Self>) {
        self.dispatcher
            .send(InputEvent::Improvement(ImprovementEvent::from(event)))
            .context("Failed to send Improvement event")
            .unwrap();
    }
}

impl<C, T> Handler<LogEvent> for Tui<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn handle(&mut self, event: &LogEvent, _: &EventContext<'_, Self>) {
        self.dispatcher
            .send(InputEvent::Log(event.0, event.1.clone()))
            .context("Failed to send LogEvent")
            .unwrap();
    }
}

impl<C, T> Handler<StreamEvent<StreamSnapshot>> for Tui<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn handle(&mut self, event: &StreamEvent<StreamSnapshot>, _: &EventContext<'_, Self>) {
        self.dispatcher
            .send(InputEvent::Stream(event.0.clone()))
            .context("Failed to send StreamSnapshot event")
            .unwrap();
    }
}

impl<C, T> Handler<StreamEvent<StreamClosed>> for Tui<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn handle(&mut self, event: &StreamEvent<StreamClosed>, _: &EventContext<'_, Self>) {
        self.dispatcher
            .send(InputEvent::Stream(event.0.0.clone()))
            .context("Failed to send StreamClosed event")
            .unwrap();
    }
}

impl<C, T> Drop for Tui<C, T>
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
