#[cfg(test)]
mod tests;

use std::{
    collections::{HashMap, HashSet},
    fmt,
    mem::{self, take},
    ops::{Add, Deref, DerefMut},
    usize,
};

const DEFAULT_TAG: &str = "div";

const EMPTY_ELEMENT_TAGS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

#[derive(Clone, PartialEq, Debug)]
pub enum AttrValue {
    One(String),
    Set(HashSet<String>),
}
impl<'a> Default for AttrValue {
    fn default() -> Self {
        Self::One(Default::default())
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Attrs(HashMap<String, AttrValue>);

impl Attrs {
    pub fn new() -> Self {
        Self(Default::default())
    }
}
impl Deref for Attrs {
    type Target = HashMap<String, AttrValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Attrs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ContentPart<'a> {
    Body(Body<'a>),
    Wrapper(Wrapper<'a>),
    Item(Item<'a>),
}
impl<'a> Default for ContentPart<'a> {
    fn default() -> Self {
        Self::Body(Default::default())
    }
}

type Slots = Vec<String>;
type ContentMap<'a> = HashMap<String, ContentPart<'a>>;

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
    tag: String,
    template: Option<Template<'a>>,
}
impl<'a> Component<'a> {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.into(),
            ..Default::default()
        }
    }
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
impl<'a> Item<'a> {
    pub fn new(tag: &str) -> Self {
        Self {
            wrapper: Wrapper::Custom(Component::new(tag)),
            ..Default::default()
        }
    }
}

impl From<&str> for AttrValue {
    fn from(value: &str) -> Self {
        Self::One(value.into())
    }
}

impl<const N: usize> From<[&str; N]> for AttrValue {
    fn from(values: [&str; N]) -> Self {
        Self::Set(values.map(|v| v.into()).into())
    }
}

impl<T: Into<AttrValue>, const N: usize> From<[(&str, T); N]> for Attrs {
    fn from(attrs: [(&str, T); N]) -> Self {
        attrs.into_iter().fold(Attrs::new(), |acc, x| acc + x)
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
    I: Into<ContentPart<'a>>,
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

impl Add<&AttrValue> for AttrValue {
    type Output = Self;

    fn add(self, other: &AttrValue) -> Self::Output {
        match other {
            Self::One(other_one) => match self {
                Self::One(one) => {
                    if one == *other_one {
                        Self::One(one)
                    } else {
                        Self::Set([one, other_one.clone()].into())
                    }
                }
                Self::Set(mut set) => {
                    set.insert(other_one.clone());

                    Self::Set(set)
                }
            },
            Self::Set(other_set) => {
                let mut set = match self {
                    Self::One(one) => [one].into(),
                    Self::Set(set) => set,
                };
                set.extend(other_set.iter().cloned());

                Self::Set(set)
            }
        }
    }
}

impl<N: Into<String>, V: Into<AttrValue>> Add<(N, V)> for Attrs {
    type Output = Self;

    fn add(self, entry: (N, V)) -> Self::Output {
        let mut attrs = self;
        let name = entry.0.into();
        let value = entry.1.into();

        if let Some(current_value) = attrs.get_mut(&name) {
            *current_value = take(current_value) + &value;
        } else {
            attrs.insert(name, value);
        }

        attrs
    }
}

impl Add<&Attrs> for Attrs {
    type Output = Self;

    fn add(self, other: &Self) -> Self::Output {
        let mut attrs = self;

        other.iter().for_each(|(other_name, other_value)| {
            attrs = take(&mut attrs) + (other_name, other_value.clone());
        });

        attrs
    }
}

impl<'a> Add<&Content<'a>> for Content<'a> {
    type Output = Self;

    fn add(self, other: &Content<'a>) -> Self::Output {
        match other {
            Content::Text(other_text) => Content::Text(if let Content::Text(text) = self {
                text + other_text
            } else {
                other_text.clone()
            }),

            Content::Parts(other_slots, other_parts) => {
                if let Content::Parts(mut slots, mut parts) = self {
                    slots = slots.or_else(|| other_slots.clone());

                    if let Some(other_parts) = other_parts {
                        if let Some(ref mut parts) = parts {
                            other_parts.iter().for_each(|(other_name, other_part)| {
                                if let Some(part) = parts.get_mut(other_name) {
                                    *part = mem::take(part) + other_part;
                                } else {
                                    parts.insert(other_name.clone(), other_part.clone());
                                };
                            });
                        };
                    }
                    Content::Parts(slots, parts)
                } else {
                    Content::Parts(other_slots.clone(), other_parts.clone())
                }
            }
        }
    }
}

impl<'a> Add<&Body<'a>> for Body<'a> {
    type Output = Body<'a>;

    fn add(self, other: &Body<'a>) -> Self::Output {
        Body {
            attrs: if let Some(ref other_attrs) = other.attrs {
                Some(self.attrs.unwrap_or_default() + other_attrs)
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
            wrapper: other.clone(),
            body: Some(self),
        }
    }
}

impl<'a> Add<&Item<'a>> for Body<'a> {
    type Output = Item<'a>;

    fn add(self, other: &Item<'a>) -> Self::Output {
        Item {
            wrapper: other.wrapper.clone(),
            body: Some(if let Some(ref other_body) = other.body {
                self + other_body
            } else {
                self
            }),
        }
    }
}

impl<'a> Add<&Body<'a>> for Wrapper<'a> {
    type Output = Item<'a>;

    fn add(self, other: &Body<'a>) -> Self::Output {
        Item {
            wrapper: self,
            body: Some(other.clone()),
        }
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

impl<'a> Add<&ContentPart<'a>> for ContentPart<'a> {
    type Output = Self;

    fn add(self, other: &ContentPart<'a>) -> Self::Output {
        match self {
            Self::Body(body) => match other {
                Self::Body(other_body) => Self::Body(body + other_body),
                Self::Wrapper(other_wrapper) => Self::Item(body + other_wrapper),
                Self::Item(other_item) => Self::Item(body + other_item),
            },

            Self::Wrapper(wrapper) => match other {
                Self::Body(other_body) => Self::Item(wrapper + other_body),
                _ => Self::Wrapper(wrapper),
            },

            Self::Item(item) => Self::Item(match other {
                Self::Body(other_body) => item + other_body,
                _ => item,
            }),
        }
    }
}

impl<'a> fmt::Display for AttrValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::One(value) => value.clone(),
            Self::Set(values) => values
                .iter()
                .map(|v| v.clone())
                .reduce(|a, b| format!("{a} {b}"))
                .unwrap_or_default(),
        };

        write!(f, "{value}")
    }
}

impl<'a> fmt::Display for Content<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut generated_slots = None;

        let create_id_part = |id: &str| Body {
            attrs: Some([("id", id)].into()),
            ..Default::default()
        };

        let content = match self {
            Self::Text(text) => text.into(),

            Self::Parts(slots, Some(items)) => slots
                .as_ref()
                .unwrap_or_else(|| generated_slots.insert(items.keys().cloned().collect()))
                .iter()
                .filter_map(|name| match items.get(name) {
                    Some(ContentPart::Item(item)) => {
                        Some(item.clone().add(&create_id_part(name)).to_string())
                    }
                    _ => None,
                })
                .reduce(|a, b| a + &b)
                .unwrap_or_default(),
            _ => "".into(),
        };

        write!(f, "{content}")
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
        let body = if let Some(body) = body {
            Some(if let Some(template) = template {
                &*glued_body.insert(template.clone() + body)
            } else {
                body
            })
        } else {
            template
        };

        let attrs = match body {
            Some(Body {
                attrs: Some(attrs), ..
            }) => attrs
                .iter()
                .map(|(k, v)| format!(" {k}=\"{v}\""))
                .reduce(|a, b| a + &b)
                .unwrap_or_default(),
            _ => "".into(),
        };

        let content_part = if !EMPTY_ELEMENT_TAGS.contains(&tag) {
            let content = match body {
                Some(Body {
                    content: Some(content),
                    ..
                }) => content.to_string(),
                _ => "".into(),
            };
            format!("{content}</{tag}>")
        } else {
            "".into()
        };

        write!(f, "<{tag}{attrs}>{content_part}")
    }
}
