use crate::{dashboard::Dashboard, state::MetricsTab};
use crossterm::event::{self, Event, KeyCode};
use radiate_engines::{EngineEvent, EventHandler, MetricSet, Objective, Score};
use std::{
    sync::{
        Arc,
        mpsc::{self, Sender},
    },
    time::Duration,
};

enum InputEvent {
    Tick,
    Key(KeyCode),
    EpochComplete(usize, MetricSet, Score, Objective),
}

pub struct EngineUi {
    dispatcher: Arc<Sender<InputEvent>>,
    event_listener: Option<std::thread::JoinHandle<()>>,
    key_listener: Option<std::thread::JoinHandle<()>>,
}

impl EngineUi {
    pub fn new(render_interval: Duration) -> Self {
        let (tx, rx) = mpsc::channel::<InputEvent>();
        let arc_sender = Arc::new(tx);

        let arc_sender_clone = Arc::clone(&arc_sender);

        let event_listener = Self::create_event_listener(rx, render_interval);
        let key_listner = Self::create_key_listener(arc_sender_clone);

        Self {
            dispatcher: arc_sender,
            event_listener: Some(event_listener),
            key_listener: Some(key_listner),
        }
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
                    Ok(InputEvent::Key(key)) => {
                        if key == KeyCode::Char('q') {
                            break;
                        } else if key == KeyCode::Char('t') {
                            et.set_metric_tab(MetricsTab::Time);
                        } else if key == KeyCode::Char('s') {
                            et.set_metric_tab(MetricsTab::Stats);
                        } else if key == KeyCode::Char('d') {
                            et.set_metric_tab(MetricsTab::Distributions);
                        } else if key == KeyCode::Char('f') {
                            et.toggle_tag_filter_display();
                        } else {
                            match key {
                                // new: browse rows
                                KeyCode::Down | KeyCode::Char('j') => et.move_selection_down(),
                                KeyCode::Up | KeyCode::Char('k') => et.move_selection_up(),

                                KeyCode::Char(c) => {
                                    if let Some(digit) = c.to_digit(10) {
                                        et.set_tag_filter_by_index(digit as usize);
                                    }
                                }

                                // new: toggle plotting of selected metric
                                // KeyCode::Char('p') => et.state_mut().toggle_selected_metric_plot(),
                                _ => {}
                            }
                        }
                    }
                    Ok(InputEvent::Tick) => {
                        et.render().unwrap_or_default();
                    }
                    Ok(InputEvent::EpochComplete(index, metrics, score, objective)) => {
                        et.update(metrics, score, index, objective);
                        et.try_render().unwrap_or_default();
                    }

                    Err(_) => break,
                }
            }
        })
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
}

impl Drop for EngineUi {
    fn drop(&mut self) {
        if let Some(key_listener) = self.key_listener.take() {
            key_listener.join().unwrap();
        }
        if let Some(listener) = self.event_listener.take() {
            listener.join().unwrap();
        }

        ratatui::restore();
    }
}

impl<T> EventHandler<T> for EngineUi {
    fn handle(&mut self, event: Arc<EngineEvent<T>>) {
        match event.as_ref() {
            EngineEvent::EpochComplete(index, _, metrics, score, objectives) => {
                self.dispatcher
                    .send(InputEvent::EpochComplete(
                        *index,
                        metrics.clone(),
                        score.clone(),
                        objectives.clone(),
                    ))
                    .unwrap_or_default();
            }
            EngineEvent::Stop(_, _, _, _) => {
                self.dispatcher.send(InputEvent::Tick).unwrap_or_default()
            }
            _ => {}
        }
    }
}
