use super::*;

fn create_item_1() -> Item<'static> {
    let mut item = Item::new("a");
    item.set_attr("class", "button")
        .set_attr("hreflang", "en")
        .set_content("Hello");

    item
}

#[test]
#[should_panic(expected = "Tag must not be empty string.")]
fn dont_create_wrapper_with_empty_tag() {
    Wrapper::new("");
}

#[test]
fn item_to_string() {
    let item_string = create_item_1().to_string();
    let result_1 = "<a class=\"button\" hreflang=\"en\">\"Hello\"</a>";
    let result_2 = "<a hreflang=\"en\" class=\"button\">\"Hello\"</a>";

    assert!(
        item_string == result_1 || item_string == result_2,
        "result is {item_string}, but should be {result_1} or {result_2}"
    )
}
