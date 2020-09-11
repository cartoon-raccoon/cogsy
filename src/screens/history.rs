use cursive::views::*;

#[allow(dead_code)]
pub struct History {

}

#[allow(dead_code)]
impl History {
    pub fn new() -> Self {
        History{}
    }
    pub fn build(&self) -> StackView {
        let screen = StackView::new();
        screen
    }
}