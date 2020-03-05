use tui::widgets::Text;

#[derive(Default)]
pub struct InspectionState<'a> {
    pub is_initiated: bool,
    pub is_json_format: bool,
    pub text: Option<Text<'a>>,
    pub scroll_value: u16,
}

impl<'a> InspectionState<'a> {
    pub fn new() -> Self {
        Self {
            is_initiated: false,
            is_json_format: false,
            text: None,
            scroll_value: 0,
        }
    }

    pub fn initiate(&mut self) {
        self.is_initiated = true;
    }

    pub fn close(&mut self) {
        self.is_initiated = false;
        self.is_json_format = false;
        self.text = None;
        self.scroll_value = 0;
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

    pub fn scroll_down(&mut self) {
        //TODO: count newline markers from self.text to forbid from scrolling below available text
        self.scroll_value += 1
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_value > 0 {
            self.scroll_value -= 1
        }
    }
}
