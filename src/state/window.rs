use tui::style::{Modifier, Style};
use tui::widgets::Text;

const BORDER_MARGIN: usize = 2;

#[derive(Default)]
pub struct WindowState<'a> {
    pub lines: Vec<Text<'a>>,
    pub line_is_selected: bool,
    pub selected_line: Option<Text<'a>>,
    selected_line_index: usize,
    selected_line_index_relative: usize,
    line_count: usize,
    displayed_line_amount: usize,
}

impl<'a> WindowState<'a> {
    pub fn new() -> Self {
        Self {
            lines: vec![],
            line_is_selected: false,
            selected_line: None,
            selected_line_index: 0,
            selected_line_index_relative: 0,
            line_count: 0,
            displayed_line_amount: 0,
        }
    }

    pub fn next(&mut self) {
        if self.selected_line_index > 0 {
            self.selected_line_index -= 1;
        }

        if self.selected_line_index_relative > 0 {
            self.selected_line_index_relative -= 1;
        }
    }

    pub fn previous(&mut self) {
        if self.line_count <= self.selected_line_index + 1 {
            return;
        }

        if self.line_is_selected {
            self.selected_line_index += 1;

            if self.selected_line_index < self.displayed_line_amount {
                self.selected_line_index_relative += 1;
            }
        } else {
            self.line_is_selected = true
        };
    }

    pub fn display_lines(&mut self, lines: &[Text<'a>], window_height: usize) {
        self.calculate_displayed_line_amount(window_height);
        self.calculate_relative_selected_line_index();
        self.line_count = lines.len();

        let skipped_line_amount = self.selected_line_index - self.selected_line_index_relative;

        self.lines = lines
            .iter()
            .rev()
            .skip(skipped_line_amount)
            .take(self.displayed_line_amount)
            .cloned()
            .collect();

        if self.line_is_selected {
            self.apply_selected_style();
        }
    }

    pub fn reset(&mut self) {
        self.line_is_selected = false;
        self.selected_line_index = 0;
        self.selected_line_index_relative = 0;
    }

    fn apply_selected_style(&mut self) {
        if let Text::Styled(cow, style) = &self.lines[self.selected_line_index_relative] {
            let text_value = cow.to_string();
            let style_value = *style;

            self.lines[self.selected_line_index_relative] = Text::styled(
                text_value.clone(),
                Style::default().modifier(Modifier::REVERSED),
            );

            self.selected_line = Some(Text::styled(text_value, style_value));
        }
    }

    fn calculate_displayed_line_amount(&mut self, window_height: usize) {
        self.displayed_line_amount = window_height - BORDER_MARGIN;
    }

    fn calculate_relative_selected_line_index(&mut self) {
        if self.selected_line_index_relative >= self.displayed_line_amount - 1 {
            self.selected_line_index_relative = self.displayed_line_amount - 1;
        }
    }
}
