pub mod state;

extern crate termion;

use failure::Error;
use state::{InspectionState, SearchState, TabsState, WindowState};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;
use toml::Value;

const CONFIG_FILE_NAME: &str = "config.toml";
const CONFIG_LOG_PATH_TOML_PROPERTY: &str = "log_path";
const MESSAGE_FILTERS_TOML_PROPERTY: &str = "message_filters";

pub struct App<'a> {
    pub tabs: TabsState,
    pub messages_window: WindowState<'a>,
    pub search: SearchState<'a>,
    pub inspection_window: InspectionState<'a>,
}

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
}

pub struct Config {
    pub log_path: String,
    pub message_filters: Vec<String>,
}

impl<'a> App<'a> {
    pub fn new(message_filters: Vec<String>) -> App<'a> {
        App {
            tabs: TabsState::new(message_filters),
            messages_window: WindowState::new(),
            search: SearchState::new(),
            inspection_window: InspectionState::new(),
        }
    }
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

impl Events {
    pub fn new() -> Events {
        let (tx, rx) = mpsc::channel();

        let _input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if tx.send(Event::Input(key)).is_err() {
                            return;
                        }
                    }
                }
            })
        };

        let _tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let tx = tx.clone();
                loop {
                    tx.send(Event::Tick).unwrap();
                    thread::sleep(Duration::from_millis(100));
                }
            })
        };

        Events { rx }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}

impl Config {
    pub fn new() -> Result<Config, Error> {
        let config = std::fs::read_to_string(CONFIG_FILE_NAME)
            .expect("Failed loading config file")
            .parse::<Value>()
            .expect("Failed loading config values");

        let log_path = config[CONFIG_LOG_PATH_TOML_PROPERTY]
            .as_str()
            .expect("Failed loading config value log_path");

        let message_filters = config[MESSAGE_FILTERS_TOML_PROPERTY]
            .clone()
            .try_into::<Vec<String>>()
            .expect("Failed loading config value captured_events");

        Ok(Config {
            log_path: log_path.to_string(),
            message_filters,
        })
    }
}
