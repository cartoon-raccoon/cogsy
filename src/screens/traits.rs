use cursive::views::StackView;

pub trait Screen {
    fn new() -> Self;
    fn build(&self) -> StackView;
}