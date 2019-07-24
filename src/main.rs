extern crate termion;
extern crate toml;
extern crate tui;

mod ui;

use std::fs::File;
use std::io;
use std::io::BufReader;
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
use ui::{Event, Events, TabsState};

const CONFIG_FILE_NAME: &str = "config.toml";
const CONFIG_LOG_PATH_TOML_PROPERTY: &str = "log_path";
const MESSAGE_TYPES_TOML_PROPERTY: &str = "message_types";

struct App<'a> {
    tabs: TabsState<'a>,
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
    let mut captured_messages: Vec<Vec<String>> = vec![];
    let mut all_messages: Vec<String> = vec![];

    for _ in 0..message_types.len() {
        captured_messages.push(vec![]);
    }

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

                let events = if app.tabs.index < message_types.len() {
                    &captured_messages[app.tabs.index]
                } else {
                    &all_messages
                };

                let events = events.iter().rev().map(|evt| {
                    Text::styled(
                        evt,
                        Style::default().fg(Color::Indexed((app.tabs.index + 1) as u8)),
                    )
                });

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

        read_log(
            &mut reader,
            &message_types,
            &mut captured_messages,
            &mut all_messages,
        );
    }

    fn read_log(
        reader: &mut BufReader<File>,
        message_types: &[String],
        captured_messages: &mut Vec<Vec<String>>,
        all_messages: &mut Vec<String>,
    ) {
        loop {
            let message = reader
                .read_line()
                .expect("Failed reading file buffer")
                .unwrap();

            if message.is_empty() {
                break;
            }

            all_messages.push(message.clone());

            for (index, message_type) in message_types.iter().enumerate() {
                if message.contains(message_type) {
                    captured_messages[index].push(message.clone());
                }
            }
        }
    }
}
