use tui::style::{Modifier, Style};
use tui::widgets::Text;

const BORDER_MARGIN: usize = 2;

pub struct WindowState<'a> {
    pub lines: Vec<Text<'a>>,
    pub line_is_selected: bool,
    pub selected_line: Option<Text<'a>>,
    height: usize,
    selected_line_index: usize,
    line_count: usize,
}

impl<'a> WindowState<'a> {
    pub fn new() -> Self {
        Self {
            lines: vec![],
            height: 0,
            line_is_selected: false,
            selected_line_index: 0,
            selected_line: None,
            line_count: 0,
        }
    }

    pub fn next(&mut self) {
        if self.selected_line_index > 0 {
            self.selected_line_index -= 1;
        }
    }

    pub fn previous(&mut self) {
        if self.line_count <= self.selected_line_index + 1 {
            return;
        }

        if self.line_is_selected {
            self.selected_line_index += 1
        } else {
            self.line_is_selected = true
        };
    }

    pub fn display_lines(&mut self, lines: &[Text<'a>], window_height: usize) {
        self.height = window_height - BORDER_MARGIN;
        self.line_count = lines.len();

        let skipped_line_amount = if self.height > self.selected_line_index {
            0
        } else {
            self.selected_line_index - self.height + 1
        };

        let mut lines: Vec<Text<'a>> = lines
            .iter()
            .rev()
            .skip(skipped_line_amount)
            .take(self.height)
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
