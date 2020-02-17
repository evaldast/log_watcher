use tui::widgets::Text;

pub struct InspectionState<'a> {
    pub is_initiated: bool,
    pub is_json_format: bool,
    pub text: Option<Text<'a>>,
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
