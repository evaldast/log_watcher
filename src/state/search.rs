use rayon::prelude::*;
use tui::widgets::Text;

pub struct SearchState<'a> {
    pub results: Vec<Text<'a>>,
    pub is_initiated: bool,
    pub input: String,
    pub should_filter: bool,
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

            // if self.input.len() > 1 {
            //     self.results = self
            //         .results
            //         .iter()
            //         .filter(|line| match line {
            //             Text::Styled(cow, _) => {
            //                 cow.to_string().to_lowercase().contains(search_input)
            //             }
            //             _ => false,
            //         })
            //         .cloned()
            //         .collect();
            // } else {
            //     self.results = lines
            //         .iter()
            //         .filter(|line| match line {
            //             Text::Styled(cow, _) => {
            //                 cow.to_string().to_lowercase().contains(search_input)
            //             }
            //             _ => false,
            //         })
            //         .cloned()
            //         .collect();
            // }
        }

        &self.results
    }
}
