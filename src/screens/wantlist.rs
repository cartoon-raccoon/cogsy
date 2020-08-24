use cursive::Cursive;
use cursive::views::*;
use cursive::view::SizeConstraint;
use cursive::traits::*;

pub struct Wantlist {

}

impl Wantlist {
    pub fn new() -> Self {
        Wantlist{}
    }
    pub fn build(&self) -> StackView {
        let main_screen = Panel::new(ResizedView::new(
            SizeConstraint::Full,
            SizeConstraint::Full,
            ScrollView::new(
                SelectView::<String>::new()
                .with_name("wantlist"))
        ));
        let mut screen = StackView::new();
        screen.add_fullscreen_layer(main_screen);

        screen
    }
}

pub fn add_to_list(s: &mut Cursive, name: &str, to_add: &str) {
    s.call_on_name(name, |view: &mut SelectView<String>| {
        view.add_item_str(to_add);
    });
}