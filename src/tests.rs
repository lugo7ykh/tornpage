use std::collections::HashMap;

use crate::{Body, Component, Content, Item, Wrapper};

fn create_item_1() -> Item<'static> {
    Item {
        wrapper: Wrapper::Custom(Component {
            tag: "a".into(),
            template: None,
        }),
        body: Body {
            attrs: HashMap::from(
                [("hreflang", "en")].map(|(key, value)| (key.into(), value.into())),
            )
            .into(),
            content: Content::from("Hello").into(),
        }
        .into(),
    }
}
fn create_body_1() -> Body<'static> {
    Body {
        attrs: HashMap::from([("href", "/hello")].map(|(key, value)| (key.into(), value.into())))
            .into(),
        content: Content::from(" World!").into(),
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
                attrs: HashMap::from(
                    [("hreflang", "en"), ("href", "/hello")]
                        .map(|(key, value)| (key.into(), value.into())),
                )
                .into(),
                content: Content::from("Hello World! World!").into(),
            }
            .into(),
        }
    );
}

#[test]
fn render_item_part_1() {
    let rendered_item = create_item_1().to_string();

    assert_eq!(rendered_item, "<a hreflang=\"en\">Hello</a>")
}

#[test]
fn render_glued_item() {
    let glued_item = create_item_1() + &create_body_1();
    let rendered_item = glued_item.to_string();

    assert!(
        rendered_item == "<a hreflang=\"en\" href=\"/hello\">Hello World!</a>"
            || rendered_item == "<a href=\"/hello\" hreflang=\"en\">Hello World!</a>"
    )
    // == "<a class=\"pretty button\" hreflang=\"en\" href=\"/hello\">Hello World!</a>"
    // || rendered_item
    //     == "<a class=\"pretty button\" href=\"/hello\" hreflang=\"en\">Hello World!</a>"
    // || rendered_item
    //     == "<a class=\"button pretty\" hreflang=\"en\" href=\"/hello\">Hello World!</a>"
    // || rendered_item
    //     == "<a class=\"button pretty\" href=\"/hello\" hreflang=\"en\">Hello World!</a>"
}
