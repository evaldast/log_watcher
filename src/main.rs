extern crate chrono;
extern crate termion;
extern crate toml;
extern crate tui;

use chrono::prelude::*;
use failure::Error;
use log_watcher::{App, Config, Event, Events};
use std::fs::File;
use std::io::{self, stdout, BufReader, Stdout, Write};
use termion::cursor::Goto;
use termion::event::Key;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Corner, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, Paragraph, Tabs, Text, Widget};
use tui::Terminal;

const ALL_MESSAGES_INDEX: usize = 0;

fn main() -> Result<(), failure::Error> {
    let config = Config::new()?;
    let file = File::open(config.log_path).expect("Failed opening file");
    let events = Events::new();

    let mut reader = BufReader::new(file);
    let mut app = App::new(&config.message_filters);
    let mut terminal = setup_terminal()?;
    let mut captured_messages: Vec<Vec<Text>> = vec![];

    for _ in 0..=config.message_filters.len() {
        captured_messages.push(vec![]);
    }

    loop {
        read_user_input(&events, &mut app)?;
        read_log(&mut reader, &config.message_filters, &mut captured_messages);
        draw_ui(&mut terminal, &mut app, &captured_messages)?;

        if app.search.is_initiated && !app.inspection_window.is_initiated {
            terminal.show_cursor()?;

            write!(
                terminal.backend_mut(),
                "{}",
                Goto(7 + &app.search.get_cursor_location(), 7)
            )?;

            io::stdout().flush().ok();
        } else {
            terminal.hide_cursor()?;
        }
    }
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
        let current_time_string = Utc::now().format("%Y-%m-%d-%H:%M:%S").to_string();        

        let constraints = if app.inspection_window.is_initiated {
            [Constraint::Percentage(100)].as_ref()
        } else {
            [Constraint::Length(3), Constraint::Percentage(100)].as_ref()
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(f.size());

        Block::default()
            .style(Style::default().bg(Color::White))
            .render(&mut f, chunks[0]);

        if app.inspection_window.is_initiated {
            app.inspection_window
                .inspect(app.messages_window.selected_line.as_ref().unwrap());

            Paragraph::new(
                [app.inspection_window.text.as_ref().unwrap()]
                    .iter()
                    .cloned(),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(&current_time_string),
            )
            .alignment(Alignment::Left)
            .wrap(!app.inspection_window.is_json_format)
            .scroll(app.inspection_window.scroll_value)
            .render(&mut f, chunks[0]);

            return;
        }

        if app.search.is_initiated {
            Paragraph::new([Text::raw(&app.search.input)].iter())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(&current_time_string),
                )
                .alignment(Alignment::Left)
                .wrap(true)
                .render(&mut f, chunks[0]);
        } else {
            Tabs::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(&current_time_string),
                )
                .titles(&app.tabs.titles)
                .select(app.tabs.index)
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(Style::default().fg(Color::Yellow))
                .render(&mut f, chunks[0]);
        }

        if app.search.is_initiated && !app.search.input.is_empty() {
            app.messages_window.display_lines(
                &app.search.get_results(&captured_messages[app.tabs.index]),
                chunks[1].height as usize,
            );
        } else {
            app.messages_window.display_lines(
                &captured_messages[app.tabs.index],
                chunks[1].height as usize,
            );
        };

        List::new(app.messages_window.lines.iter().cloned())
            .block(Block::default().borders(Borders::ALL).title("Messages"))
            .start_corner(Corner::BottomLeft)
            .render(&mut f, chunks[1]);
    })
}

fn read_user_input(events: &Events, app: &mut App) -> Result<(), Error> {
    //TODO: Group and cleanup
    if let Event::Input(input) = events.next()? {
        match input {
            Key::Char(c)
                if app.search.is_initiated && !app.inspection_window.is_initiated && c != '\n' =>
            {
                app.search.add_input(c);
                app.messages_window.reset();
            }
            Key::Backspace if app.search.is_initiated && !app.inspection_window.is_initiated => {
                app.search.remove_input_backspace();
                app.messages_window.reset();
            }
            Key::Delete if app.search.is_initiated && !app.inspection_window.is_initiated => {
                app.search.remove_input_delete();
                app.messages_window.reset();
            }
            Key::Left if app.search.is_initiated && !app.inspection_window.is_initiated => {
                app.search.cursor_move_left()
            }
            Key::Right if app.search.is_initiated && !app.inspection_window.is_initiated => {
                app.search.cursor_move_right()
            }
            Key::Esc if app.inspection_window.is_initiated => app.inspection_window.close(),
            Key::Esc if app.search.is_initiated => {
                app.search.close();
                app.messages_window.reset()
            }
            Key::Esc if app.messages_window.line_is_selected => app.messages_window.reset(),
            Key::Char('q') => failure::bail!("User called Quit"),
            Key::Right => switch_tab(app, true),
            Key::Left => switch_tab(app, false),
            Key::Up if app.inspection_window.is_initiated => app.inspection_window.scroll_up(),
            Key::Up => app.messages_window.previous(),
            Key::Down if app.inspection_window.is_initiated => app.inspection_window.scroll_down(),
            Key::Down => app.messages_window.next(),
            Key::Char('f') => {
                app.messages_window.reset();
                app.search.initiate();
            }
            Key::Char('\n') => app.inspection_window.initiate(),
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
