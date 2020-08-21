use cursive::views::*;
use cursive::traits::*;
use cursive::Cursive;
use cursive::view::SizeConstraint;

pub struct Collection {}

#[allow(dead_code)]
impl Collection {
    pub fn new() -> Self {
        Collection{}
    }
    pub fn build(&self) -> StackView {
        let album_list = ResizedView::new(
            SizeConstraint::Full, 
            SizeConstraint::Full, 
            LinearLayout::horizontal()
            .child(SelectView::<String>::new()
                .with_name("albumlist")));
        
        let commandline = EditView::new();
        let layout = LinearLayout::vertical()
            .child(album_list)
            .child(commandline);
        let mut screen = StackView::new();
        screen.add_fullscreen_layer(layout);
        screen
    }
}

pub fn add_to_list(s: &mut Cursive, name: &str) {
    s.call_on_name("albumlist", |view: &mut SelectView<String>| {
        view.add_item_str(name);
    });
}