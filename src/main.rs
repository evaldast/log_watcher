extern crate termion;
extern crate toml;
extern crate tui;

mod ui;

use failure::Error;
use std::fs::File;
use std::io::{self, stdout, BufReader, Stdout, Write};
use termion::cursor::Goto;
use termion::event::Key;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use toml::Value;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Corner, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, Paragraph, Tabs, Text, Widget};
use tui::Terminal;
use ui::{Event, Events, SearchState, TabsState, WindowState};
use unicode_width::UnicodeWidthStr;

const CONFIG_FILE_NAME: &str = "config.toml";
const CONFIG_LOG_PATH_TOML_PROPERTY: &str = "log_path";
const MESSAGE_FILTERS_TOML_PROPERTY: &str = "message_filters";
const ALL_MESSAGES_INDEX: usize = 0;

struct App<'a> {
    tabs: TabsState<'a>,
    messages_window: WindowState<'a>,
    search: SearchState,
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
        messages_window: WindowState::new(),
        search: SearchState::new(),
    };

    let mut terminal = setup_terminal()?;
    let events = Events::new();
    let mut captured_messages: Vec<Vec<Text>> = vec![];

    for _ in 0..=message_types.len() {
        captured_messages.push(vec![]);
    }

    loop {
        read_user_input(&events, &mut app)?;
        read_log(&mut reader, &message_types, &mut captured_messages);
        draw_ui(&mut terminal, &mut app, &captured_messages)?;

        if app.search.is_initiated {
            terminal.show_cursor()?;

            write!(
                terminal.backend_mut(),
                "{}",
                Goto(7 + app.search.input.width() as u16, 7)
            )
            .unwrap();

            io::stdout().flush().ok();
        } else {
            terminal.hide_cursor()?;
        }
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
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

fn draw_ui<'a>(
    terminal: &mut Terminal<TermionBackend<AlternateScreen<RawTerminal<Stdout>>>>,
    app: &mut App<'a>,
    captured_messages: &[Vec<Text<'a>>],
) -> Result<(), std::io::Error> {
    terminal.draw(|mut f| {
        let is_alternate_view = app.messages_window.line_is_selected;
        let is_search = app.search.is_initiated;

        let constraints = if is_alternate_view {
            [
                Constraint::Length(3),
                Constraint::Percentage(80),
                Constraint::Percentage(20),
            ]
            .as_ref()
        } else {
            [Constraint::Length(3), Constraint::Percentage(100)].as_ref()
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints(constraints)
            .split(f.size());

        Block::default()
            .style(Style::default().bg(Color::White))
            .render(&mut f, chunks[0]);

        if is_search {
            Paragraph::new([Text::raw(&app.search.input)].iter())
                .block(Block::default().borders(Borders::ALL).title("Search Input"))
                .alignment(Alignment::Left)
                .wrap(true)
                .render(&mut f, chunks[0]);
        } else {
            Tabs::default()
                .block(Block::default().borders(Borders::ALL).title("Tabs"))
                .titles(&app.tabs.titles)
                .select(app.tabs.index)
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(Style::default().fg(Color::Yellow))
                .render(&mut f, chunks[0]);
        }

        app.messages_window.display_lines(
            &captured_messages[app.tabs.index],
            chunks[1].height as usize,
            &app.search.input,
        );

        List::new(app.messages_window.lines.iter().cloned())
            .block(Block::default().borders(Borders::ALL).title("Messages"))
            .start_corner(Corner::BottomLeft)
            .render(&mut f, chunks[1]);

        if app.messages_window.line_is_selected {
            let selected_line = app.messages_window.selected_line.as_ref().unwrap();

            Paragraph::new([selected_line].iter().cloned())
                .block(Block::default().borders(Borders::ALL).title("Selected"))
                .alignment(Alignment::Left)
                .wrap(true)
                .render(&mut f, chunks[2]);
        }
    })
}

fn read_user_input(events: &Events, app: &mut App) -> Result<(), Error> {
    if let Event::Input(input) = events.next()? {
        match input {
            Key::Char(i) if app.search.is_initiated && i != '\n' => app.search.input.push(i),
            Key::Backspace if app.search.is_initiated => {
                app.search.input.pop();
            }
            Key::Esc if app.search.is_initiated => app.search.close(),
            Key::Char('q') => failure::bail!("User called Quit"),
            Key::Right => switch_tab(app, true),
            Key::Left => switch_tab(app, false),
            Key::Up => app.messages_window.previous(),
            Key::Down => app.messages_window.next(),
            Key::Char('f') => {
                app.messages_window.selected_line_index = 0;
                app.messages_window.line_is_selected = false;
                app.search.search();
                },
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

    while let Some(message) = reader.read_line().expect("Failed reading line") {
        if message.is_empty() {
            break;
        }

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

fn switch_tab(app: &mut App, is_next: bool) {
    app.messages_window.reset();

    if is_next {
        app.tabs.next();
    } else {
        app.tabs.previous();
    }
}
