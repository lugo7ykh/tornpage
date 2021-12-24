use std::collections::{HashMap, HashSet};

use super::{Content, Item, Render};
use super::{Glue, ItemPart};

fn create_item_part_1() -> ItemPart<'static> {
    ItemPart {
        tag: Some("a".into()),
        class: Some(HashSet::from(["button"].map(|name| name.into()))),
        attrs: Some(HashMap::from(
            [("hreflang", "en")].map(|(key, value)| (key.into(), value.into())),
        )),
        content: Some("Hello".into()),
    }
}
fn create_item_part_2() -> ItemPart<'static> {
    ItemPart {
        tag: None,
        class: Some(HashSet::from(["pretty"].map(|name| name.into()))),
        attrs: Some(HashMap::from(
            [("href", "/hello")].map(|(key, value)| (key.into(), value.into())),
        )),
        content: Some(" World!".into()),
    }
}

#[test]
fn glue_item_parts() {
    let item_part_1 = create_item_part_1();
    let item_part_2 = create_item_part_2();

    let glued_item = ItemPart::glue(&vec![&item_part_1, &item_part_2, &item_part_2]);

    assert_eq!(
        glued_item,
        Some(ItemPart {
            tag: Some("a".into()),
            class: Some(HashSet::from(["pretty", "button"].map(|name| name.into()))),
            attrs: Some(HashMap::from(
                [("hreflang", "en"), ("href", "/hello")]
                    .map(|(key, value)| (key.into(), value.into()))
            )),
            content: Some("Hello World! World!".into()),
        })
    );
}

#[test]
fn render_item_part_1() {
    let rendered_item = Item::from(&create_item_part_1()).render();

    assert_eq!(
        rendered_item,
        "<a class=\"button\" hreflang=\"en\">Hello</a>"
    )
}

#[test]
fn render_item_part_2() {
    let rendered_item = Item::from(&create_item_part_2()).render();

    assert_eq!(rendered_item, "")
}

#[test]
fn render_glued_item() {
    let glued_item =
        ItemPart::glue(&vec![&create_item_part_1(), &create_item_part_2()]).expect("can't glue");
    let rendered_item = Item::from(&glued_item).render();

    assert!(
        rendered_item
            == "<a class=\"pretty button\" hreflang=\"en\" href=\"/hello\">Hello World!</a>"
            || rendered_item
                == "<a class=\"pretty button\" href=\"/hello\" hreflang=\"en\">Hello World!</a>"
            || rendered_item
                == "<a class=\"button pretty\" hreflang=\"en\" href=\"/hello\">Hello World!</a>"
            || rendered_item
                == "<a class=\"button pretty\" href=\"/hello\" hreflang=\"en\">Hello World!</a>"
    )
}
