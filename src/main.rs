extern crate termion;
extern crate toml;
extern crate tui;

mod ui;

use failure::Error;
use std::fs::File;
use std::io::{stdout, BufReader, Stdout};
use termion::event::Key;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use toml::Value;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph, Tabs, Text, Widget};
use tui::Terminal;
use ui::{Event, Events, TabsState};

const CONFIG_FILE_NAME: &str = "config.toml";
const CONFIG_LOG_PATH_TOML_PROPERTY: &str = "log_path";
const MESSAGE_FILTERS_TOML_PROPERTY: &str = "message_filters";
const ALL_MESSAGES_INDEX: usize = 0;

struct App<'a> {
    tabs: TabsState<'a>,
}

struct Config {
    log_path: String,
    message_filters: Vec<String>,
}

fn main() -> Result<(), failure::Error> {
    let config = load_config()?;
    let log_path = config.log_path;
    let message_types = config.message_filters;

    let file = File::open(log_path).expect("Failed opening file");
    let mut reader = BufReader::new(file);
    let tabs: Vec<&str> = message_types.iter().map(AsRef::as_ref).collect();

    let mut app = App {
        tabs: TabsState::new(&tabs),
    };

    let mut terminal = setup_terminal()?;
    let events = Events::new();
    let mut captured_messages: Vec<Vec<Text>> = vec![];

    for _ in 0..=message_types.len() {
        captured_messages.push(vec![]);
    }

    loop {
        draw_ui(&mut terminal, &app, &captured_messages)?;
        read_user_input(&events, &mut app)?;
        read_log(&mut reader, &message_types, &mut captured_messages);
    }
}

fn load_config() -> Result<Config, Error> {
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

fn setup_terminal() -> Result<Terminal<TermionBackend<AlternateScreen<RawTerminal<Stdout>>>>, Error>
{
    let stdout = stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    Ok(terminal)
}

fn draw_ui(
    terminal: &mut Terminal<TermionBackend<AlternateScreen<RawTerminal<Stdout>>>>,
    app: &App,
    captured_messages: &[Vec<Text>],
) -> Result<(), std::io::Error> {
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

        Paragraph::new(captured_messages[app.tabs.index].iter().rev())
            .block(Block::default().borders(Borders::ALL).title("Messages"))
            .alignment(Alignment::Left)
            .wrap(true)
            .render(&mut f, chunks[1]);
    })
}

fn read_user_input(events: &Events, app: &mut App) -> Result<(), Error> {
    if let Event::Input(input) = events.next()? {
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
    captured_messages: &mut [Vec<Text>],
) {
    use termion::input::TermRead;

    while let Some(mut message) = reader.read_line().expect("Failed reading line") {
        if message.is_empty() {
            break;
        }

        message.push('\n');

        capture_message(message_types, captured_messages, &message);
    }
}

fn capture_message(message_types: &[String], captured_messages: &mut [Vec<Text>], message: &str) {
    let mut message_captured = false;

    for (index, message_type) in message_types.iter().enumerate() {
        if message.contains(message_type) {
            let styled = Text::styled(
                message.to_string(),
                Style::default().fg(Color::Indexed((index + 1) as u8)),
            );

            captured_messages[index + 1].push(styled.clone());
            captured_messages[ALL_MESSAGES_INDEX].push(styled);

            message_captured = true;
        }
    }

    if !message_captured {
        let styled = Text::styled(message.to_string(), Style::default().fg(Color::White));

        captured_messages[ALL_MESSAGES_INDEX].push(styled);
    }
}
