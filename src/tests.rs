use crate::{Attrs, Body, Component, Item, Wrapper};

fn create_item_1() -> Item<'static> {
    Item {
        wrapper: Wrapper::Custom(Component {
            tag: "a".into(),
            template: None,
        }),
        body: Body {
            attrs: Some(Attrs::new() + ("class", "button") + ("hreflang", "en")),
            content: Some("Hello".into()),
        }
        .into(),
    }
}
fn create_body_1() -> Body<'static> {
    Body {
        attrs: Some(Attrs::new() + ("class", "pretty") + ("href", "/hello")),
        content: Some(" World!".into()),
    }
}

#[test]
fn glue_body_to_item() {
    let item = create_item_1();
    let body = create_body_1();

    let glued_item = item + &body + &create_body_1();

    assert_eq!(
        glued_item,
        Item {
            wrapper: Wrapper::Custom(Component {
                tag: "a".into(),
                template: None,
            }),
            body: Body {
                attrs: Some(
                    Attrs::new()
                        + ("class", ["pretty", "button"])
                        + ("hreflang", "en")
                        + ("href", "/hello")
                ),
                content: Some("Hello World! World!".into()),
            }
            .into(),
        }
    );
}

#[test]
fn render_item_part_1() {
    let rendered_item = create_item_1().to_string();

    assert!(
        rendered_item == "<a class=\"button\" hreflang=\"en\">Hello</a>"
            || rendered_item == "<a hreflang=\"en\" class=\"button\">Hello</a>"
    )
}
