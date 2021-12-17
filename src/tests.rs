use std::collections::{HashMap, HashSet};

use super::{Content, Item, Render};
use super::{ItemPart, Merge};

fn create_item_part_1() -> ItemPart<'static> {
    ItemPart {
        tag: Some("a".into()),
        class: Some(HashSet::from(["button"].map(|name| name.into()))),
        attrs: Some(HashMap::from(
            [("hreflang", "en")].map(|(key, value)| (key.into(), value.into())),
        )),
        content: Some(Content::from("Hello")),
        slots: Some(vec!["label".into(), "info".into()]),
    }
}
fn create_item_part_2() -> ItemPart<'static> {
    ItemPart {
        tag: None,
        class: Some(HashSet::from(["pretty"].map(|name| name.into()))),
        attrs: Some(HashMap::from(
            [("href", "/hello")].map(|(key, value)| (key.into(), value.into())),
        )),
        content: Some(Content::from(" World!")),
        slots: None,
    }
}

#[test]
fn merge_item_parts() {
    let item_part_1 = create_item_part_1();
    let item_part_2 = create_item_part_2();

    let merged_item = ItemPart::merge(&vec![&item_part_1, &item_part_2, &item_part_2]);

    assert_eq!(
        merged_item,
        Some(ItemPart {
            tag: Some("a".into()),
            class: Some(HashSet::from(["pretty", "button"].map(|name| name.into()))),
            attrs: Some(HashMap::from(
                [("hreflang", "en"), ("href", "/hello")]
                    .map(|(key, value)| (key.into(), value.into()))
            )),
            content: Some(Content::from("Hello World! World!")),
            slots: Some(vec!["label".into(), "info".into()]),
        })
    );
}

#[test]
fn render_item_part_1() {
    let rendered_item = Item::Part(&create_item_part_1()).render();

    assert_eq!(
        rendered_item,
        "<a class=\"button\" hreflang=\"en\">Hello</a>"
    )
}

#[test]
fn render_item_part_2() {
    let rendered_item = Item::Part(&create_item_part_2()).render();

    assert_eq!(rendered_item, "")
}

#[test]
fn render_merged_item() {
    let merged_item =
        ItemPart::merge(&vec![&create_item_part_1(), &create_item_part_2()]).expect("can't merge");
    let rendered_item = Item::Part(&merged_item).render();

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
