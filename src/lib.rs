#[cfg(test)]
mod tests;

use std::{
    collections::{HashMap, HashSet},
    fmt,
    mem::take,
    ops::{Add, Deref, DerefMut},
    usize,
};

const EMPTY_COMPONENT_TAGS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];
fn is_empty_component_tag(tag: &str) -> bool {
    EMPTY_COMPONENT_TAGS.contains(&tag)
}

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

#[derive(Clone, PartialEq, Debug)]
pub enum Layout {
    Fixed(Vec<String>),
    Extensible(Vec<String>),
}
impl<'a> Default for Layout {
    fn default() -> Self {
        Self::Extensible(Default::default())
    }
}
impl Deref for Layout {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Fixed(layout) | Self::Extensible(layout) => layout,
        }
    }
}
type PartsMap<'a> = HashMap<String, ContentPart<'a>>;

#[derive(Clone, PartialEq, Debug)]
pub enum Content<'a> {
    Text(String),
    Parts(Option<Layout>, Option<PartsMap<'a>>),
}
impl<'a> Content<'a> {
    pub fn new() -> Self {
        Self::Parts(None, None)
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Body<'a> {
    attrs: Option<Attrs>,
    content: Option<Content<'a>>,
}
impl<'a> Body<'a> {
    fn is_empty(&self) -> bool {
        self.attrs.is_none() && self.content.is_none()
    }
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
impl<'a> Template<'a> {
    fn get(&self) -> &Body {
        match self {
            &Template::Ref(template) | Template::Custom(template) => template,
        }
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Component<'a> {
    tag: String,
    template: Option<Template<'a>>,
}
impl<'a> Component<'a> {
    /// Create a new Component.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the tag is empty.
    pub fn new(tag: &str) -> Self {
        if tag == "" {
            panic!("Component tag must not be empty.");
        };

        Self {
            tag: tag.to_lowercase(),

            template: if is_empty_component_tag(tag) {
                Some(Template::Custom(Body::from(Content::from([]))))
            } else {
                None
            },
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
impl<'a> Wrapper<'a> {
    fn get(&self) -> &Component {
        match self {
            &Wrapper::Ref(component) | Wrapper::Custom(component) => component,
        }
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

impl<N: Into<String>, V: Into<AttrValue>> FromIterator<(N, V)> for Attrs {
    fn from_iter<T: IntoIterator<Item = (N, V)>>(entries: T) -> Self {
        entries
            .into_iter()
            .fold(Attrs::new(), |attrs, entry| attrs + entry)
    }
}

impl<'a> From<&str> for Content<'a> {
    fn from(text: &str) -> Self {
        Self::Text(text.into())
    }
}

impl<'a, const N: usize> From<[&str; N]> for Content<'a> {
    fn from(layout: [&str; N]) -> Self {
        let content = layout.into_iter().fold(Content::new(), |content, slot| {
            content + (slot, ContentPart::Body(Default::default()))
        });

        if let Content::Parts(Some(Layout::Extensible(layout)), parts) = content {
            Content::Parts(Some(Layout::Fixed(layout)), parts)
        } else {
            Content::Parts(Some(Layout::Fixed(vec![])), None)
        }
    }
}

impl<'a, S: Into<String>, P: Into<ContentPart<'a>>> FromIterator<(S, P)> for Content<'a> {
    fn from_iter<T: IntoIterator<Item = (S, P)>>(entries: T) -> Self {
        entries
            .into_iter()
            .fold(Content::new(), |content, entry| content + entry)
    }
}

impl<'a> From<Attrs> for Body<'a> {
    fn from(attrs: Attrs) -> Self {
        Self {
            attrs: Some(attrs),
            ..Default::default()
        }
    }
}
impl<'a> From<Content<'a>> for Body<'a> {
    fn from(content: Content<'a>) -> Self {
        Self {
            content: Some(content),
            ..Default::default()
        }
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

impl Add<(&String, &AttrValue)> for Attrs {
    type Output = Self;

    fn add(self, (name, value): (&String, &AttrValue)) -> Self::Output {
        let mut attrs = self;

        if let Some(current_value) = attrs.get_mut(name) {
            *current_value = take(current_value) + value;
        } else {
            attrs.insert(name.clone(), value.clone());
        }

        attrs
    }
}

impl<N: Into<String>, V: Into<AttrValue>> Add<(N, V)> for Attrs {
    type Output = Self;

    fn add(self, (name, value): (N, V)) -> Self::Output {
        self + (&name.into(), &value.into())
    }
}

impl Add<&Attrs> for Attrs {
    type Output = Self;

    fn add(self, other: &Self) -> Self::Output {
        other.iter().fold(self, |attrs, entry| attrs + entry)
    }
}

impl<'a> Add<&str> for Content<'a> {
    type Output = Self;

    fn add(self, text: &str) -> Self::Output {
        if let Content::Text(current_text) = self {
            Self::Text(current_text + text)
        } else {
            self
        }
    }
}

impl<'a> Add<(&String, &ContentPart<'a>)> for Content<'a> {
    type Output = Self;

    fn add(self, (slot, part): (&String, &ContentPart<'a>)) -> Self::Output {
        if let Content::Parts(mut layout, mut parts) = self {
            if let Layout::Extensible(layout) = layout.get_or_insert_with(Default::default) {
                let parts = parts.get_or_insert_with(Default::default);

                if !parts.contains_key(slot) {
                    parts.insert(slot.clone(), Default::default());
                    layout.push(slot.clone())
                }
            }

            if let Some(ref mut parts) = parts {
                if let Some(current_part) = parts.get_mut(slot) {
                    *current_part = take(current_part) + part;
                }
            }

            Self::Parts(layout, parts)
        } else {
            self
        }
    }
}

impl<'a, S: Into<String>, P: Into<ContentPart<'a>>> Add<(S, P)> for Content<'a> {
    type Output = Self;

    fn add(self, (slot, part): (S, P)) -> Self::Output {
        self + (&slot.into(), &part.into())
    }
}

impl<'a> Add<&Content<'a>> for Content<'a> {
    type Output = Self;

    fn add(self, other: &Content<'a>) -> Self::Output {
        match other {
            Content::Text(text) => self + text.as_str(),

            Content::Parts(Some(layout), Some(parts)) => {
                layout.iter().fold(self, |content, slot| {
                    content + (slot, parts.get(slot).unwrap())
                })
            }

            _ => self,
        }
    }
}

impl<'a> Add<Attrs> for Body<'a> {
    type Output = Self;

    fn add(self, other_attrs: Attrs) -> Self::Output {
        Self {
            attrs: Some(if let Some(attrs) = self.attrs {
                attrs + &other_attrs
            } else {
                other_attrs
            }),
            ..self
        }
    }
}

impl<'a> Add<Content<'a>> for Body<'a> {
    type Output = Self;

    fn add(self, other_content: Content<'a>) -> Self::Output {
        Self {
            content: Some(if let Some(content) = self.content {
                content + &other_content
            } else {
                other_content
            }),
            ..self
        }
    }
}

impl<'a> Add<&Body<'a>> for Body<'a> {
    type Output = Body<'a>;

    fn add(self, other: &Body<'a>) -> Self::Output {
        Body {
            attrs: if let Some(ref attrs) = other.attrs {
                Some(
                    self.attrs
                        .map_or_else(|| attrs.clone(), |current_attrs| current_attrs + attrs),
                )
            } else {
                self.attrs
            },
            content: if let Some(ref content) = other.content {
                Some(self.content.map_or_else(
                    || content.clone(),
                    |current_content| current_content + content,
                ))
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

impl<'a> Add<Attrs> for Item<'a> {
    type Output = Self;

    fn add(self, other_attrs: Attrs) -> Self::Output {
        Self {
            body: Some(self.body.unwrap_or_default() + other_attrs),
            ..self
        }
    }
}

impl<'a> Add<Content<'a>> for Item<'a> {
    type Output = Self;

    fn add(self, other_content: Content<'a>) -> Self::Output {
        Self {
            body: Some(self.body.unwrap_or_default() + other_content),
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

impl<'a> fmt::Display for Attrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let attrs = self
            .iter()
            .map(|(k, v)| format!(" {k}=\"{v}\""))
            .reduce(|a, b| a + &b)
            .unwrap_or_default();

        write!(f, "{attrs}")
    }
}

impl<'a> fmt::Display for Content<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let create_id_part = |id: &str| Body::from(Attrs::new() + ("id", id));

        let content = match self {
            Self::Text(text) => text.into(),

            Self::Parts(Some(layout), Some(parts)) => layout
                .iter()
                .filter_map(|name| match parts.get(name) {
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
        let Component { tag, template } = wrapper.get();
        let template = template.as_ref().map(|template| template.get());

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
            }) => attrs.to_string(),
            _ => "".into(),
        };

        let content_part = if !is_empty_component_tag(tag) {
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
