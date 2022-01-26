use crate::{Attrs, Body, Content, Item};

fn create_item_1() -> Item<'static> {
    Item::new("a")
        + (Attrs::new() + ("class", "button") + ("hreflang", "en"))
        + Content::from("Hello")
}

fn create_body_1() -> Body<'static> {
    Body::from(Attrs::new() + ("class", "pretty") + ("href", "/hello")) + Content::from(" World!")
}

#[test]
fn add_body_to_item() {
    let glued_item = create_item_1() + &create_body_1() + &create_body_1();

    assert_eq!(
        glued_item,
        Item::new("a")
            + (Attrs::new()
                + ("class", ["pretty", "button"])
                + ("hreflang", "en")
                + ("href", "/hello"))
            + Content::from("Hello World! World!"),
    );
}

#[test]
fn display_item_1() {
    let rendered_item = create_item_1().to_string();

    assert!(
        rendered_item == "<a class=\"button\" hreflang=\"en\">Hello</a>"
            || rendered_item == "<a hreflang=\"en\" class=\"button\">Hello</a>"
    )
}
