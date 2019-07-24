extern crate termion;
extern crate toml;
extern crate tui;

mod ui;

use failure::Error;
use std::fs::File;
use std::io::{stdout, BufReader, Stdout};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
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

struct Config {
    log_path: String,
    message_types: Vec<String>,
}

fn main() -> Result<(), failure::Error> {
    let config = load_config()?;
    let log_path = config.log_path;
    let message_types = config.message_types;

    let file = File::open(log_path).expect("Failed opening file");
    let mut reader = BufReader::new(file);

    let tabs: Vec<&str> = message_types.iter().map(AsRef::as_ref).collect();

    let mut app = App {
        tabs: TabsState::new(tabs),
    };

    let mut terminal = setup_terminal()?;

    let events = Events::new();
    let mut captured_messages: Vec<Vec<String>> = vec![];

    for _ in 0..=message_types.len() {
        captured_messages.push(vec![]);
    }

    loop {
        terminal.draw(|mut f| {
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

            let events = captured_messages[app.tabs.index].iter().rev().map(|evt| {
                Text::styled(
                    evt,
                    Style::default().fg(Color::Indexed((app.tabs.index + 1) as u8)),
                )
            });

            List::new(events)
                .block(Block::default().borders(Borders::ALL).title("Messages"))
                .start_corner(Corner::BottomLeft)
                .render(&mut f, chunks[1]);
        })?;

        match read_user_input(&events, &mut app) {
            Ok(_) => {}
            Err(_) => break,
        }

        read_log(&mut reader, &message_types, &mut captured_messages);
    }

    Ok(())
}

fn load_config() -> Result<Config, Error> {
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

    Ok(Config {
        log_path: log_path.to_string(),
        message_types,
    })
}

fn setup_terminal() -> Result<Terminal<TermionBackend<AlternateScreen<RawTerminal<Stdout>>>>, Error>
{
    let stdout = stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    Ok(terminal)
}

fn read_user_input(events: &Events, app: &mut App) -> Result<(), Error> {
    if let Event::Input(input) = events.next().unwrap() {
        match input {
            Key::Char('q') => {
                failure::bail!("User called Quit");
            }
            Key::Right => app.tabs.next(),
            Key::Left => app.tabs.previous(),
            _ => {}
        }
    };

    Ok(())
}

fn read_log(
    reader: &mut BufReader<File>,
    message_types: &[String],
    captured_messages: &mut Vec<Vec<String>>,
) {
    loop {
        let message = reader
            .read_line()
            .expect("Failed reading file buffer")
            .unwrap();

        if message.is_empty() {
            break;
        }

        captured_messages[0].push(message.clone());

        for (index, message_type) in message_types.iter().enumerate() {
            if message.contains(message_type) {
                captured_messages[index + 1].push(message.clone());
            }
        }
    }
}
