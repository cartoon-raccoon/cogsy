use cursive::views::*;

pub struct History {

}

impl History {
    pub fn new() -> Self {
        History{}
    }
    pub fn build(&self) -> StackView {
        let screen = StackView::new();
        screen
    }
}