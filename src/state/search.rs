use rayon::prelude::*;
use tui::widgets::Text;
use unicode_width::UnicodeWidthStr;

pub struct SearchState<'a> {
    pub results: Vec<Text<'a>>,
    pub is_initiated: bool,
    pub input: String,
    pub should_filter: bool,
    cursor_location: usize,
}

impl<'a> SearchState<'a> {
    pub fn new() -> Self {
        Self {
            results: vec![],
            is_initiated: false,
            input: String::new(),
            should_filter: false,
            cursor_location: 0,
        }
    }

    pub fn initiate(&mut self) {
        self.is_initiated = true;
    }

    pub fn close(&mut self) {
        self.is_initiated = false;
        self.input = String::new();
        self.cursor_location = 0;
    }

    pub fn get_results(&mut self, lines: &[Text<'a>]) -> &[Text<'a>] {
        //TODO: think about using LRU when user is deleting input and detach from UI thread/papralelise filtering
        if self.should_filter {
            self.should_filter = false;

            let search_input = &self.input.to_lowercase();
            self.results = lines
                .par_iter()
                .filter(|line| match line {
                    Text::Styled(cow, _) => cow.to_string().to_lowercase().contains(search_input),
                    _ => false,
                })
                .cloned()
                .collect();
        }

        &self.results
    }

    pub fn add_input(&mut self, character: char) {
        let mut new_input = String::new();

        new_input.push_str(&self.input[0..self.cursor_location]);
        new_input.push(character);
        new_input.push_str(&self.input[self.cursor_location..]);

        self.input = new_input;
        self.cursor_move_right();
        self.should_filter = true;
    }

    pub fn remove_input_backspace(&mut self) {
        if self.input.len() > 0 as usize {
            let mut new_input = String::new();

            new_input.push_str(&self.input[0..self.cursor_location - 1]);
            new_input.push_str(&self.input[self.cursor_location..]);

            self.input = new_input;
            self.cursor_move_left();
            self.should_filter = true;
        }
    }

    pub fn remove_input_delete(&mut self) {
        if self.input.len() > 0 as usize {
            let mut new_input = String::new();

            new_input.push_str(&self.input[0..self.cursor_location]);
            new_input.push_str(&self.input[self.cursor_location + 1..]);

            self.input = new_input;
            self.should_filter = true;
        }
    }

    pub fn cursor_move_left(&mut self) {
        if self.cursor_location == 0 && self.input.len() > 0 {
            self.cursor_location = self.input.len() - 1;
        } else if self.cursor_location > 0 {
            self.cursor_location -= 1;
        }
    }

    pub fn cursor_move_right(&mut self) {
        if self.input.len() > self.cursor_location {
            self.cursor_location += 1;
        }
    }

    pub fn get_cursor_location(&self) -> u16 {
        self.input[0..self.cursor_location].width() as u16
    }
}
