extern crate termion;
extern crate toml;
extern crate tui;

use std::fs::File;
use std::io;
use std::io::BufReader;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use toml::Value;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Corner, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, Tabs, Text, Widget};
use tui::Terminal;

const CONFIG_FILE_NAME: &str = "config.toml";
const CONFIG_LOG_PATH_TOML_PROPERTY: &str = "log_path";
const MESSAGE_TYPES_TOML_PROPERTY: &str = "message_types";

struct App<'a> {
    tabs: TabsState<'a>,
}

pub struct TabsState<'a> {
    titles: Vec<&'a str>,
    index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    input_handle: thread::JoinHandle<()>,
    tick_handle: thread::JoinHandle<()>,
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();

        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if tx.send(Event::Input(key)).is_err() {
                            return;
                        }
                        if key == config.exit_key {
                            return;
                        }
                    }
                }
            })
        };

        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let tx = tx.clone();
                loop {
                    tx.send(Event::Tick).unwrap();
                    thread::sleep(config.tick_rate);
                }
            })
        };

        Events {
            rx,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}

fn main() {
    let config = std::fs::read_to_string(CONFIG_FILE_NAME)
        .expect("Failed loading config file")
        .parse::<Value>()
        .expect("Failed loading config values");

    let log_path = config[CONFIG_LOG_PATH_TOML_PROPERTY]
        .as_str()
        .expect("Failed loading config value log_path");

    let message_types = config[MESSAGE_TYPES_TOML_PROPERTY]
        .clone()
        .try_into::<Vec<String>>()
        .expect("Failed loading config value captured_events");

    let file = File::open(log_path).expect("Failed opening file");
    let mut reader = BufReader::new(file);

    let mut tabs: Vec<&str> = message_types.iter().map(AsRef::as_ref).collect();
    tabs.push("All");

    let mut app = App {
        tabs: TabsState::new(tabs),
    };

    let stdout = io::stdout().into_raw_mode().unwrap();
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.hide_cursor().unwrap();

    let events = Events::new();
    let mut captured_messages: Vec<Vec<String>> = vec![vec![], vec![], vec![]];

    loop {
        terminal
            .draw(|mut f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(5)
                    .constraints([Constraint::Length(3), Constraint::Percentage(100)].as_ref())
                    .split(size);

                Block::default()
                    .style(Style::default().bg(Color::White))
                    .render(&mut f, chunks[0]);
                Tabs::default()
                    .block(Block::default().borders(Borders::ALL).title("Tabs"))
                    .titles(&app.tabs.titles)
                    .select(app.tabs.index)
                    .style(Style::default().fg(Color::Cyan))
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .render(&mut f, chunks[0]);

                let events = captured_messages[app.tabs.index]
                    .iter()
                    .rev()
                    .map(|evt| Text::styled(evt, Style::default().fg(Color::Yellow)));

                List::new(events)
                    .block(Block::default().borders(Borders::ALL).title("Messages"))
                    .start_corner(Corner::BottomLeft)
                    .render(&mut f, chunks[1]);
            })
            .unwrap();

        if let Event::Input(input) = events.next().unwrap() {
            match input {
                Key::Char('q') => {
                    break;
                }
                Key::Right => app.tabs.next(),
                Key::Left => app.tabs.previous(),
                _ => {}
            }
        };

        loop {
            let event = reader
                .read_line()
                .expect("Failed reading file buffer")
                .unwrap();

            if event.is_empty() {
                break;
            }

            for (index, event_type) in message_types.iter().enumerate() {
                if event.contains(event_type) {
                    captured_messages[index].push(event);

                    break;
                }
            }
        }
    }
}
