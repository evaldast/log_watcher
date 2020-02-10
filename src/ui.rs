extern crate termion;
extern crate tui;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;
use tui::style::{Modifier, Style};
use tui::widgets::Text;

const BORDER_MARGIN: usize = 2;

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

pub struct WindowState<'a> {
    pub lines: Vec<Text<'a>>,
    pub height: usize,
    pub line_is_selected: bool,
    pub selected_line_index: usize,
    pub selected_line: Option<Text<'a>>,
}

pub struct SearchState<'a> {
    pub results: Vec<Text<'a>>,
    pub is_initiated: bool,
    pub input: String,
    pub should_filter: bool,
}

pub struct InspectionState<'a> {
    pub is_initiated: bool,
    pub is_json_format: bool,
    pub text: Option<Text<'a>>,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: &[&'a str]) -> Self {
        Self {
            titles: [&["All"][..], titles].concat(),
            index: 0,
        }
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

impl<'a> WindowState<'a> {
    pub fn new() -> Self {
        Self {
            lines: vec![],
            height: 0,
            line_is_selected: false,
            selected_line_index: 0,
            selected_line: None,
        }
    }

    pub fn next(&mut self) {
        if self.selected_line_index > 0 {
            self.selected_line_index -= 1;
        }
    }

    pub fn previous(&mut self) {
        if self.line_is_selected {
            self.selected_line_index += 1
        } else {
            self.line_is_selected = true
        };
    }

    pub fn display_lines(&mut self, lines: &[Text<'a>], window_height: usize) {
        self.height = window_height - BORDER_MARGIN;

        let skipped_line_amount = if self.height > self.selected_line_index {
            0
        } else {
            self.selected_line_index - self.height + 1
        };

        let displayed_line_amount = skipped_line_amount + self.height + 1;

        let mut lines: Vec<Text<'a>> = lines
            .iter()
            .rev()
            .skip(skipped_line_amount)
            .take(displayed_line_amount)
            .cloned()
            .collect();

        let selected_line_index = self.selected_line_index - skipped_line_amount;

        if self.line_is_selected {
            if let Text::Styled(cow, style) = &lines[selected_line_index] {
                let text_value = cow.to_string();
                let style_value = *style;

                lines[selected_line_index] = Text::styled(
                    text_value.clone(),
                    Style::default().modifier(Modifier::REVERSED),
                );

                self.selected_line = Some(Text::styled(text_value, style_value));
            }
        }

        self.lines = lines;
    }

    pub fn reset(&mut self) {
        self.line_is_selected = false;
        self.selected_line_index = 0;
    }
}

impl<'a> SearchState<'a> {
    pub fn new() -> Self {
        Self {
            results: vec![],
            is_initiated: false,
            input: String::new(),
            should_filter: false,
        }
    }

    pub fn initiate(&mut self) {
        self.is_initiated = true;
    }

    pub fn close(&mut self) {
        self.is_initiated = false;
        self.input = String::new();
    }

    pub fn get_results(&mut self, lines: &[Text<'a>]) -> &Vec<Text<'a>> {
        if self.should_filter {
            self.should_filter = false;

            self.results = lines
                .iter()
                .filter(|line| match line {
                    Text::Styled(cow, _) => cow
                        .to_string()
                        .to_lowercase()
                        .contains(&self.input.to_lowercase()),
                    _ => false,
                })
                .cloned()
                .collect();
        }

        &self.results
    }
}

impl<'a> InspectionState<'a> {
    pub fn new() -> Self {
        Self {
            is_initiated: false,
            is_json_format: false,
            text: None,
        }
    }

    pub fn initiate(&mut self) {
        self.is_initiated = true;
    }

    pub fn close(&mut self) {
        self.is_initiated = false;
        self.is_json_format = false;
        self.text = None;
    }

    pub fn inspect(&mut self, text: &Text) {
        if let Text::Styled(cow, style) = text {
            let json_opening_brace_index = match cow.find('{') {
                Some(i) => i,
                None => {
                    self.text = Some(Text::styled(cow.to_string(), *style));

                    return;
                }
            };

            let json_closing_brace_index: usize = {
                let mut result = 0;
                for (i, c) in cow.chars().enumerate() {
                    if c == '}' {
                        result = i;
                    }
                }

                result + 1
            };

            let potential_json = &cow[json_opening_brace_index..json_closing_brace_index];

            match serde_json::from_str::<serde_json::Value>(potential_json) {
                Ok(json) => {
                    let text_to_display = format!(
                        "{}\n{}\n{}",
                        &cow[..json_opening_brace_index].to_string(),
                        serde_json::to_string_pretty(&json).unwrap(),
                        &cow[json_closing_brace_index..].to_string()
                    );

                    self.text = Some(Text::styled(text_to_display, *style));

                    self.is_json_format = true;
                }
                Err(_) => {
                    self.text = Some(Text::styled(cow.to_string(), *style));
                }
            };
        }
    }
}

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            tick_rate: Duration::from_millis(100),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
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
                    thread::sleep(config.tick_rate);
                }
            })
        };

        Events { rx }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}
