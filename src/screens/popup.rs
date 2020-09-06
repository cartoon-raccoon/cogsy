use cursive::views::*;

use crate::app::Release;

//TODO: Build this properly
pub fn build(release: Release) -> Canvas<String> {
    let state = String::from("Hello");
    let canvas = Canvas::new(state)
                    .with_draw(|text: &String, printer| {
                        // Simply print our string
                        printer.print((0,0), text);
                    });
    canvas
}