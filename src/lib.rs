#[cfg(test)]
mod tests;

use std::{
    collections::{HashMap, HashSet},
    ops::Add,
};

const EMPTY_ELEMENT_TAGS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

type Slots = Vec<String>;
type ItemList<'a> = HashMap<String, Item<'a>>;

#[derive(Clone, PartialEq, Debug)]
pub enum Content<'a> {
    Text(String),
    Items(Option<Slots>, Option<ItemList<'a>>),
}
impl<'a> Default for Content<'a> {
    fn default() -> Self {
        Content::Items(None, None)
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct ItemPart<'a> {
    pub tag: Option<String>,
    pub class: Option<HashSet<String>>,
    pub attrs: Option<HashMap<String, String>>,
    pub content: Option<Content<'a>>,
}
type ItemParts<'a> = Vec<&'a ItemPart<'a>>;

#[derive(Clone, PartialEq, Debug)]
pub enum Item<'a> {
    Part(&'a ItemPart<'a>),
    Parts(ItemParts<'a>),
}

pub struct PageOptions {
    title_separator: String,
}
pub struct PagePart<'a> {
    pub title: Option<String>,
    pub options: Option<PageOptions>,
    pub head: Option<Content<'a>>,
    pub body: Option<Content<'a>>,
}
type PageParts<'a> = Vec<&'a PagePart<'a>>;
pub enum Page<'a> {
    Part(&'a PagePart<'a>),
    Parts(PageParts<'a>),
}

trait Glue {
    fn glue(parts: &Vec<&Self>) -> Option<Self>
    where
        Self: Sized;
}
pub trait Render {
    fn render(&self) -> String;
}

impl<'a> From<&str> for Content<'a> {
    fn from(text: &str) -> Self {
        Content::Text(text.into())
    }
}
impl<'a, S: Into<String>, I: Into<Item<'a>>> From<HashMap<S, I>> for Content<'a> {
    fn from(items: HashMap<S, I>) -> Self {
        Content::Items(
            None,
            Some(
                items
                    .into_iter()
                    .map(|(s, i)| (s.into(), i.into()))
                    .collect(),
            ),
        )
    }
}
impl<'a, N: Into<String>> From<Vec<N>> for Content<'a> {
    fn from(slots: Vec<N>) -> Self {
        Content::Items(Some(slots.into_iter().map(|n| n.into()).collect()), None)
    }
}
impl<'a, T: Into<String>> From<(Vec<T>, HashMap<String, Item<'a>>)> for Content<'a> {
    fn from((slots, items): (Vec<T>, HashMap<String, Item<'a>>)) -> Self {
        Content::glue(&vec![&slots.into(), &items.into()]).unwrap_or_default()
    }
}

impl<'a> Item<'a> {
    fn tag(&self) -> Option<&String> {
        match self {
            Item::Part(item) => item.tag.as_ref(),
            Item::Parts(parts) => match parts
                .iter()
                .rev()
                .skip_while(|part| part.tag.is_none())
                .next()
            {
                Some(part) => part.tag.as_ref(),
                _ => None,
            },
        }
    }
}

impl<'a> From<&'a ItemPart<'a>> for Item<'a> {
    fn from(part: &'a ItemPart) -> Self {
        Item::Part(part)
    }
}
impl<'a> From<ItemParts<'a>> for Item<'a> {
    fn from(parts: ItemParts<'a>) -> Self {
        Item::Parts(parts)
    }
}

impl<'a, T: Into<Item<'a>>> Add<T> for Item<'a> {
    type Output = Self;

    fn add(self, other: T) -> Self::Output {
        match self {
            Item::Part(part) => match other.into() {
                Item::Part(other_part) => {
                    if part != other_part {
                        Item::from(vec![part, other_part])
                    } else {
                        Item::from(part)
                    }
                }
                Item::Parts(other_parts) => {
                    let mut result = vec![part];
                    result.extend(other_parts);
                    Item::from(result)
                }
            },
            Item::Parts(parts) => {
                let mut result = parts;
                match other.into() {
                    Item::Part(other_part) => result.push(other_part),
                    Item::Parts(other_parts) => result.extend(other_parts),
                }
                Item::from(result)
            }
        }
    }
}

impl<'a> Glue for Content<'a> {
    fn glue(parts: &Vec<&Self>) -> Option<Self> {
        let mut content = None;

        for part in parts.iter().rev() {
            match part {
                Content::Text(part_text) => {
                    if let Content::Text(text) = content.get_or_insert_with(|| "".into()) {
                        text.insert_str(0, part_text);
                    } else {
                        break;
                    }
                }
                Content::Items(part_slots, part_items) => {
                    if let Content::Items(slots, items) =
                        content.get_or_insert_with(|| Content::Items(None, None))
                    {
                        if slots.is_none() && part_slots.is_some() {
                            *slots = part_slots.clone();
                        }
                        if let Some(part_items) = part_items {
                            let items = items.get_or_insert_with(|| HashMap::new());

                            for (id, part_item) in part_items {
                                let item = items.remove(id);
                                items.insert(
                                    id.into(),
                                    if let Some(item) = item {
                                        part_item.clone() + item
                                    } else {
                                        part_item.clone()
                                    },
                                );
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        content
    }
}

fn calc_class_capacity(parts: &ItemParts) -> usize {
    parts
        .iter()
        .map(|part| match part.class {
            Some(ref class) => class.len(),
            None => 0,
        })
        .sum()
}
fn calc_attrs_capacity(parts: &ItemParts) -> usize {
    parts
        .iter()
        .map(|part| match part.attrs {
            Some(ref attrs) => attrs.len(),
            None => 0,
        })
        .sum()
}

impl<'a> Glue for ItemPart<'a> {
    fn glue(parts: &Vec<&Self>) -> Option<Self> {
        let mut tag = None;
        let mut class = None;
        let mut attrs = None;
        let mut content_parts = None;

        for part in parts.iter().rev() {
            if let Some(ref part_class) = part.class {
                class
                    .get_or_insert_with(|| HashSet::with_capacity(calc_class_capacity(parts)))
                    .extend(part_class.clone().into_iter());
            }
            if let Some(ref part_attrs) = part.attrs {
                attrs
                    .get_or_insert_with(|| HashMap::with_capacity(calc_attrs_capacity(parts)))
                    .extend(part_attrs.clone().into_iter());
            }
            if let Some(ref part_content) = part.content {
                content_parts
                    .get_or_insert_with(|| Vec::with_capacity(parts.len()))
                    .push(part_content);
            }
            if part.tag.is_some() {
                tag = part.tag.clone();

                break;
            }
        }

        if tag.is_none() && class.is_none() && attrs.is_none() && content_parts.is_none() {
            None
        } else {
            if let Some(ref mut class) = class {
                class.shrink_to_fit();
            }
            if let Some(ref mut attrs) = attrs {
                attrs.shrink_to_fit();
            }
            let content = match content_parts {
                Some(mut content_parts)
                    if tag.is_none()
                        || !EMPTY_ELEMENT_TAGS.contains(&tag.as_ref().unwrap().as_str()) =>
                {
                    content_parts.reverse();
                    Content::glue(&content_parts)
                }
                _ => None,
            };

            Some(ItemPart {
                tag,
                class,
                attrs,
                content,
            })
        }
    }
}

impl<'a> Render for Content<'a> {
    fn render(&self) -> String {
        let create_id_part = |id: &str| ItemPart {
            attrs: Some(HashMap::from([("id".into(), id.into())])),
            ..Default::default()
        };
        match self {
            Self::Text(text) => text.clone(),
            Self::Items(Some(slots), Some(items)) => items
                .iter()
                .map(|(id, item)| item.clone().add(&create_id_part(id)).render())
                .reduce(|a, b| a + &b)
                .unwrap_or_default(),
            _ => Default::default(),
        }
    }
}

impl<'a> Render for Item<'a> {
    fn render(&self) -> String {
        let mut glued_item;

        let ItemPart {
            tag,
            class,
            attrs,
            content,
            ..
        } = match self {
            Item::Part(part) => *part,
            Item::Parts(item_parts) => {
                glued_item = ItemPart::glue(item_parts);
                glued_item.get_or_insert_with(Default::default)
            }
        };
        if tag.is_none() {
            return "".into();
        }

        let tag = match tag {
            Some(text) if text != "" => text,
            _ => "div",
        };
        let class = match class {
            Some(class) => format!(
                " class=\"{}\"",
                class
                    .iter()
                    .map(|n| n.clone())
                    .reduce(|a, b| a + &format!(" {}", b))
                    .unwrap_or_default(),
            ),
            _ => "".into(),
        };
        let attrs = match attrs {
            Some(attrs) => attrs
                .iter()
                .map(|(k, v)| format!(" {}=\"{}\"", k, v))
                .reduce(|a, b| a + &b)
                .unwrap_or_default(),
            _ => "".into(),
        };

        let start_tag = format!("<{}{}{}>", tag, class, attrs);
        let (content, end_tag) = if !EMPTY_ELEMENT_TAGS.contains(&tag) {
            (
                match content {
                    Some(content) => content.render(),
                    _ => "".into(),
                },
                format!("</{}>", tag),
            )
        } else {
            Default::default()
        };

        format!("{}{}{}", start_tag, content, end_tag)
    }
}

impl<'a> Render for Page<'a> {
    fn render(&self) -> String {
        format!("")
    }
}
