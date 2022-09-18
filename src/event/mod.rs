//! # event
//!
//! Contains methods to handle input events

use crossterm::event::{
    self,
    Event,
};
use std::{
    sync::mpsc,
    time::{
        Duration,
        Instant,
    },
    thread,
};

/// Used to determine if a key event occurred or not
// pub enum AppEvent {
//     Tick,
//     Key(KeyEvent),
// }

pub struct EventHandler {
    // Channel to send events
    _sender: mpsc::Sender<Option<Event>>,
    // Channel to receive events
    receiver: mpsc::Receiver<Option<Event>>,
    // Thread to handle events
    _handler: thread::JoinHandle<()>,
}

// Delay for reading events in milliseconds
const DELAY: u64 = 250;

impl EventHandler {
    /// Constructs a new EventHandler, and the thread channel to send/receive
    /// events
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let tick_len = Duration::from_millis(DELAY);
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_len
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_len);

                    if event::poll(timeout).expect("No events available") {
                        match event::read().expect("Couldn't read event") {
                            event::Event::Key(e) => sender.send(Some(Event::Key(e))),
                            _ => Ok(())
                        }.expect("Failed to send event")
                    }

                    if last_tick.elapsed() >= tick_len {
                        sender.send(None)
                            .expect("Couldn't send tick event");
                        last_tick = Instant::now();
                    }
                }
            })
        };
        Self { _sender: sender, receiver, _handler: handler }
    }

    /// Check the next event from the handler thread
    pub fn next(&self) -> Result<Option<Event>, Box<dyn std::error::Error>> {
        let next = self.receiver.recv()?;
        Ok(next)
    }
}
