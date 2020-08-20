use cursive::views::*;
use cursive::traits::*;

pub struct Collection {}

#[allow(dead_code)]
impl Collection {
    pub fn new() -> Self {
        Collection{}
    }
    pub fn build(&self) -> StackView {
        let mut album_list = LinearLayout::horizontal()
            .child(SelectView::<String>::new()
                .item("Hello", String::from("Hello"))
                .item("World", String::from("World"))
                .with_name("albumlist"));
        let mut commandline = EditView::new();
        let mut layout = LinearLayout::vertical()
            .child(album_list)
            .child(commandline);
        let mut screen = StackView::new();
        screen.add_fullscreen_layer(layout);
        screen
    }
}