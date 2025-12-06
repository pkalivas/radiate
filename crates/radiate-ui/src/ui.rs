use crate::{dashboard::Dashboard, state::MetricsTab};
use crossterm::event::{self, Event, KeyCode};
use radiate_engines::{
    Chromosome, Engine, EngineIterator, Generation, GeneticEngine, MetricSet, Objective, Score,
    error::RadiateResult,
};
use std::{
    sync::{
        Arc,
        mpsc::{self, Sender},
    },
    time::Duration,
};

enum InputEvent {
    Key(KeyCode),
    EngineStart,
    EngineStop,
    EpochComplete(usize, MetricSet, Score, Objective),
}

pub struct EngineUi<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    inner: GeneticEngine<C, T>,
    dispatcher: Arc<Sender<InputEvent>>,
    event_listener: Option<std::thread::JoinHandle<()>>,
    key_listener: Option<std::thread::JoinHandle<()>>,
}

impl<C, T> EngineUi<C, T>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync + 'static,
{
    pub fn new(inner: GeneticEngine<C, T>, render_interval: Duration) -> Self {
        let (tx, rx) = mpsc::channel::<InputEvent>();
        let arc_sender = Arc::new(tx);

        let arc_sender_clone = Arc::clone(&arc_sender);

        let event_listener = create_event_listener(rx, render_interval);
        let key_listner = create_key_listener(arc_sender_clone);

        Self {
            inner,
            dispatcher: arc_sender,
            event_listener: Some(event_listener),
            key_listener: Some(key_listner),
        }
    }

    pub fn iter(self) -> impl Iterator<Item = Generation<C, T>> {
        EngineIterator::new(self)
    }
}

impl<C, T> Engine for EngineUi<C, T>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync + 'static,
{
    type Epoch = Generation<C, T>;

    fn next(&mut self) -> RadiateResult<Self::Epoch> {
        let current = self.inner.next()?;
        if current.index() == 1 {
            self.dispatcher.send(InputEvent::EngineStart).unwrap();
        }

        self.dispatcher
            .send(InputEvent::EpochComplete(
                current.index(),
                current.metrics().clone(),
                current.score().clone(),
                current.objective().clone(),
            ))
            .unwrap();

        Ok(current)
    }
}

impl<C, T> Drop for EngineUi<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.dispatcher.send(InputEvent::EngineStop).unwrap();

        if let Some(key_listener) = self.key_listener.take() {
            key_listener.join().unwrap();
        }

        if let Some(listener) = self.event_listener.take() {
            listener.join().unwrap();
        }
    }
}

fn create_key_listener(
    arc_sender_clone: Arc<mpsc::Sender<InputEvent>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    match key.code {
                        KeyCode::Char('q') => {
                            arc_sender_clone
                                .send(InputEvent::Key(KeyCode::Char('q')))
                                .unwrap();
                            break;
                        }
                        KeyCode::Char(c) => {
                            arc_sender_clone
                                .send(InputEvent::Key(KeyCode::Char(c)))
                                .unwrap();
                        }
                        KeyCode::Up | KeyCode::Down => {
                            arc_sender_clone.send(InputEvent::Key(key.code)).unwrap();
                        }

                        _ => {}
                    }
                }
            }
        }
    })
}

fn create_event_listener(
    dispatcher: mpsc::Receiver<InputEvent>,
    render_interval: Duration,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut et = Dashboard::new(render_interval);

        loop {
            let rec = dispatcher.recv();
            match rec {
                Ok(InputEvent::Key(key)) => match key {
                    KeyCode::Char('q') => break,

                    KeyCode::Char('f') => et.toggle_tag_filter_display(),

                    KeyCode::Char('t') => et.set_metric_tab(MetricsTab::Time),
                    KeyCode::Char('s') => et.set_metric_tab(MetricsTab::Stats),
                    KeyCode::Char('d') => et.set_metric_tab(MetricsTab::Distributions),

                    KeyCode::Char('c') => et.toggle_metric_charts_display(),
                    KeyCode::Char('m') => et.toggle_metric_means_display(),

                    KeyCode::Down | KeyCode::Char('j') => et.move_selection_down(),
                    KeyCode::Up | KeyCode::Char('k') => et.move_selection_up(),

                    KeyCode::Char(c) => {
                        if let Some(digit) = c.to_digit(10) {
                            et.set_tag_filter_by_index(digit as usize);
                        }
                    }

                    _ => {}
                },
                Ok(InputEvent::EpochComplete(index, metrics, score, objective)) => {
                    et.update(metrics, score, index, objective);
                    et.try_render().unwrap_or_default();
                }
                Ok(InputEvent::EngineStart) => {
                    et.toggle_is_running();
                }
                Ok(InputEvent::EngineStop) => {
                    et.toggle_is_running();
                    et.render().unwrap_or_default();
                }
                Err(_) => {
                    break;
                }
            }
        }

        ratatui::restore();
    })
}
