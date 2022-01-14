#[cfg(test)]
mod tests;

use std::{
    collections::{HashMap, HashSet},
    fmt, mem,
    ops::Add,
    usize,
};

const DEFAULT_TAG: &str = "div";

const EMPTY_ELEMENT_TAGS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

#[derive(Clone, PartialEq, Debug)]
pub enum PagePart<'a> {
    Item(Item<'a>),
    Wrapper(Wrapper<'a>),
    Body(Body<'a>),
}
impl<'a> Default for PagePart<'a> {
    fn default() -> Self {
        Self::Body(Default::default())
    }
}

type AttrName = String;

#[derive(Clone, PartialEq, Debug)]
enum AttrValue {
    One(String),
    Set(HashSet<String>),
}
impl<'a> Default for AttrValue {
    fn default() -> Self {
        Self::One(Default::default())
    }
}
type Attrs = HashMap<AttrName, AttrValue>;

type Tag = String;
type Slots = Vec<String>;
type ContentMap<'a> = HashMap<String, PagePart<'a>>;

#[derive(Clone, PartialEq, Debug)]
pub enum Content<'a> {
    Text(String),
    Parts(Option<Slots>, Option<ContentMap<'a>>),
}
impl<'a> Default for Content<'a> {
    fn default() -> Self {
        Self::Parts(None, None)
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Body<'a> {
    attrs: Option<Attrs>,
    content: Option<Content<'a>>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Template<'a> {
    Ref(&'a Body<'a>),
    Custom(Body<'a>),
}
impl<'a> Default for Template<'a> {
    fn default() -> Self {
        Self::Custom(Default::default())
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Component<'a> {
    tag: Tag,
    template: Option<Template<'a>>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Wrapper<'a> {
    Ref(&'a Component<'a>),
    Custom(Component<'a>),
}
impl<'a> Default for Wrapper<'a> {
    fn default() -> Self {
        Self::Custom(Default::default())
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Item<'a> {
    wrapper: Wrapper<'a>,
    body: Option<Body<'a>>,
}

impl From<&str> for AttrValue {
    fn from(value: &str) -> Self {
        Self::One(value.into())
    }
}

impl<const N: usize> From<[&str; N]> for AttrValue {
    fn from(values: [&str; N]) -> Self {
        Self::Set(values.into_iter().map(|v| v.into()).collect())
    }
}

impl<'a> From<&str> for Content<'a> {
    fn from(text: &str) -> Self {
        Self::Text(text.into())
    }
}

impl<'a, S, I, const N: usize> From<[(S, I); N]> for Content<'a>
where
    S: Into<String>,
    I: Into<PagePart<'a>>,
{
    fn from(parts: [(S, I); N]) -> Self {
        Content::Parts(
            None,
            Some(
                parts
                    .into_iter()
                    .map(|(s, i)| (s.into(), i.into()))
                    .collect(),
            ),
        )
    }
}

impl<'a> Add<&Body<'a>> for Item<'a> {
    type Output = Self;

    fn add(self, other: &Body<'a>) -> Self::Output {
        Self {
            body: Some(self.body.unwrap_or_default() + other),
            ..self
        }
    }
}

impl<'a> Add<&Wrapper<'a>> for Item<'a> {
    type Output = Self;

    fn add(self, other: &Wrapper<'a>) -> Self::Output {
        Self {
            wrapper: other.to_owned(),
            ..self
        }
    }
}

impl<'a> Add<&Item<'a>> for Body<'a> {
    type Output = Item<'a>;

    fn add(self, other: &Item<'a>) -> Self::Output {
        Item {
            wrapper: other.wrapper.to_owned(),
            body: Some(if let Some(ref other_body) = other.body {
                self + other_body
            } else {
                self
            }),
        }
    }
}

impl<'a> Add<&Body<'a>> for Body<'a> {
    type Output = Body<'a>;

    fn add(self, other: &Body<'a>) -> Self::Output {
        Body {
            attrs: if let Some(ref other_attrs) = other.attrs {
                Some(
                    self.attrs
                        .unwrap_or_default()
                        .into_iter()
                        .chain(
                            other_attrs
                                .into_iter()
                                .map(|(k, v)| (k.to_owned(), v.to_owned())),
                        )
                        .collect(),
                )
            } else {
                self.attrs
            },

            content: if let Some(ref other_content) = other.content {
                Some(self.content.unwrap_or_default() + other_content)
            } else {
                self.content
            },
        }
    }
}

impl<'a> Add<&Wrapper<'a>> for Body<'a> {
    type Output = Item<'a>;

    fn add(self, other: &Wrapper<'a>) -> Self::Output {
        Item {
            wrapper: other.to_owned(),
            body: Some(self),
        }
    }
}

impl<'a> Add<&Item<'a>> for Wrapper<'a> {
    type Output = Item<'a>;

    fn add(self, other: &Item<'a>) -> Self::Output {
        Item {
            wrapper: self,
            body: other.body.to_owned(),
        }
    }
}

impl<'a> Add<&Body<'a>> for Wrapper<'a> {
    type Output = Item<'a>;

    fn add(self, other: &Body<'a>) -> Self::Output {
        Item {
            wrapper: self,
            body: Some(other.to_owned()),
        }
    }
}

impl<'a> Add<&PagePart<'a>> for PagePart<'a> {
    type Output = Self;

    fn add(self, other: &PagePart<'a>) -> Self::Output {
        match self {
            Self::Item(item) => Self::Item(match other {
                Self::Item(other_item) => other_item.to_owned(),
                Self::Body(other_body) => item + other_body,
                Self::Wrapper(other_wrapper) => item + other_wrapper,
            }),

            Self::Body(body) => match other {
                Self::Item(other_item) => Self::Item(body + other_item),
                Self::Body(other_body) => Self::Body(body + other_body),
                Self::Wrapper(other_wrapper) => Self::Item(body + other_wrapper),
            },

            Self::Wrapper(wrapper) => match other {
                Self::Item(other_item) => Self::Item(wrapper + other_item),
                Self::Body(other_body) => Self::Item(wrapper + other_body),
                Self::Wrapper(other_wrapper) => Self::Wrapper(other_wrapper.to_owned()),
            },
        }
    }
}

impl<'a> Add<&Content<'a>> for Content<'a> {
    type Output = Self;

    fn add(self, other: &Content<'a>) -> Self::Output {
        match other {
            Content::Text(other_text) => Content::Text(if let Content::Text(text) = self {
                text + other_text
            } else {
                other_text.to_owned()
            }),

            Content::Parts(other_slots, other_parts) => {
                if let Content::Parts(mut slots, mut parts) = self {
                    slots = slots.or_else(|| other_slots.to_owned());

                    if let Some(other_parts) = other_parts {
                        if let Some(ref mut parts) = parts {
                            other_parts.iter().for_each(|(other_name, other_part)| {
                                if let Some(part) = parts.get_mut(other_name) {
                                    *part = mem::take(part) + other_part;
                                } else {
                                    parts.insert(other_name.to_owned(), other_part.to_owned());
                                };
                            });
                        };
                    }
                    Content::Parts(slots, parts)
                } else {
                    Content::Parts(other_slots.to_owned(), other_parts.to_owned())
                }
            }
        }
    }
}

impl<'a> fmt::Display for AttrValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::One(value) => value.to_owned(),
            Self::Set(values) => values
                .iter()
                .map(|v| v.to_owned())
                .reduce(|a, b| a + &b)
                .unwrap_or_default(),
        };

        write!(f, "{}", value)
    }
}

impl<'a> fmt::Display for Content<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut generated_slots = None;

        let create_id_part = |id: &str| Body {
            attrs: Some(HashMap::from([("id".into(), id.into())])),
            ..Default::default()
        };

        let content = match self {
            Self::Text(text) => text.into(),

            Self::Parts(slots, Some(items)) => slots
                .as_ref()
                .unwrap_or_else(|| generated_slots.insert(items.keys().cloned().collect()))
                .iter()
                .filter_map(|name| match items.get(name) {
                    Some(PagePart::Item(item)) => {
                        Some(item.to_owned().add(&create_id_part(name)).to_string())
                    }
                    _ => None,
                })
                .reduce(|a, b| a + &b)
                .unwrap_or_default(),
            _ => "".into(),
        };

        write!(f, "{}", content)
    }
}

impl<'a> fmt::Display for Item<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Item { wrapper, body } = self;

        let Component { tag, template } = match wrapper {
            &Wrapper::Ref(component) | Wrapper::Custom(component) => component,
        };
        let tag = if tag != "" { tag } else { DEFAULT_TAG };

        let template = template.as_ref().map(|template| match template {
            &Template::Ref(template) | Template::Custom(template) => template,
        });

        let mut glued_body = None;
        let body = template
            .and_then(|template| {
                body.as_ref()
                    .and_then(|body| Some(&*glued_body.insert(template.to_owned() + body)))
                    .or(Some(template))
            })
            .or(body.as_ref());

        let attrs = match body {
            Some(Body {
                attrs: Some(attrs), ..
            }) => attrs
                .iter()
                .map(|(k, v)| format!(" {}=\"{}\"", k, v))
                .reduce(|a, b| a + &b)
                .unwrap_or_default(),
            _ => "".into(),
        };
        let start_tag = format!("<{}{}>", tag, attrs);

        let content = match body {
            Some(Body {
                content: Some(content),
                ..
            }) if !EMPTY_ELEMENT_TAGS.contains(&tag) => content.to_string(),
            _ => "".into(),
        };
        let end_tag = if !EMPTY_ELEMENT_TAGS.contains(&tag) {
            format!("</{}>", tag)
        } else {
            "".into()
        };

        write!(f, "{}{}{}", start_tag, content, end_tag)
    }
}
