#[cfg(test)]
mod tests;

use std::{
    collections::HashMap,
    fmt::{self},
    ops::{Add, Deref},
};

const DEFAULT_SLOT: &str = "";

const EMPTY_TAGS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

#[derive(Clone, PartialEq, Debug)]
pub enum AttrValue {
    String(String),
    Bool(bool),
}
impl<'a> Default for AttrValue {
    fn default() -> Self {
        Self::String(Default::default())
    }
}

type Attrs = HashMap<String, AttrValue>;

#[derive(Clone, PartialEq, Debug)]
enum ContentPart<'a> {
    Text(String),
    Item(Item<'a>),
}
type ContentParts<'a> = Vec<ContentPart<'a>>;

#[derive(Clone, PartialEq, Debug)]
struct Content<'a>(ContentParts<'a>);

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Body<'a> {
    attrs: Attrs,
    content: Option<Content<'a>>,
}

trait IsItem<'a> {
    fn tag(&self) -> &str;
    fn body(&self) -> &Body<'a>;

    fn has_attr(&self, name: &str) -> bool {
        self.body().attrs.contains_key(name)
    }
    fn attr(&'a self, name: &str) -> Option<&'a AttrValue> {
        self.body().attrs.get(name)
    }

    fn format_attr(name: &str, value: &AttrValue) -> String {
        match value.to_string().as_str() {
            "\"\"" | "false" => format!(""),
            "true" => format!(" {name}"),
            _ => format!(" {name}={value}"),
        }
    }

    fn attrs_string(&self) -> String {
        self.body()
            .attrs
            .iter()
            .fold(String::new(), |attrs, (name, value)| {
                attrs + &Self::format_attr(name, value)
            })
    }

    fn content(&self) -> Option<&Content<'a>> {
        self.body().content.as_ref()
    }
}

trait IsMutItem<'a>: IsItem<'a> {
    fn mut_body(&mut self) -> &mut Body<'a>;

    fn mut_attr(&'a mut self, name: &str) -> Option<&'a mut AttrValue> {
        self.mut_body().attrs.get_mut(name)
    }

    fn remove_attr(&mut self, name: &str) -> Option<AttrValue> {
        self.mut_body().attrs.remove(name)
    }

    fn set_attr<V: Into<AttrValue>>(&mut self, name: &str, value: V) -> &mut Self {
        self.mut_body().attrs.insert(name.into(), value.into());

        self
    }
    fn add_attr<V: Into<AttrValue>>(&mut self, name: &str, value: V) -> &mut Self {
        if let Some(current_value) = self.remove_attr(name) {
            self.set_attr(name.into(), current_value + &value.into());
        } else {
            self.set_attr(name.into(), value.into());
        }

        self
    }

    fn set_content<C: Into<Content<'a>>>(&mut self, content: C) -> &mut Self {
        self.mut_body().content.replace(content.into());

        self
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
struct Template<'a>(Body<'a>);

#[derive(Clone, PartialEq, Debug)]
enum WrapperBody<'a> {
    Body(Template<'a>),
    Ref(&'a Template<'a>),
}
impl<'a> Default for WrapperBody<'a> {
    fn default() -> Self {
        Self::Body(Default::default())
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
struct Wrapper<'a> {
    tag: String,
    body: WrapperBody<'a>,
}
impl<'a> Wrapper<'a> {
    pub fn new(tag: &str) -> Self {
        let tag = tag.trim();

        if tag == "" {
            panic!("Tag must not be empty string.")
        }

        Self {
            tag: tag.to_lowercase(),
            ..Default::default()
        }
    }

    fn slot(&self) -> Option<&String> {
        self.attr("data-slot").and_then(|value| match value {
            AttrValue::String(slot_name) => Some(slot_name),
            _ => None,
        })
    }
}
impl<'a> IsItem<'a> for Wrapper<'a> {
    fn tag(&self) -> &str {
        self.tag.as_str()
    }
    fn body(&self) -> &Body<'a> {
        match self.body {
            WrapperBody::Body(Template(ref body)) | WrapperBody::Ref(Template(ref body)) => body,
        }
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
struct Component<'a>(Wrapper<'a>);
impl<'a> Deref for Component<'a> {
    type Target = Wrapper<'a>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Item<'a> {
    wrapper: Wrapper<'a>,
    body: Body<'a>,
}
impl<'a> Item<'a> {
    pub fn new(tag: &str) -> Self {
        Self {
            wrapper: Wrapper::new(tag),
            ..Default::default()
        }
    }
}
impl<'a> Item<'a> {
    fn wrapper(&self) -> &Wrapper {
        &self.wrapper
    }
}
impl<'a> IsItem<'a> for Item<'a> {
    fn tag(&self) -> &str {
        self.wrapper().tag()
    }
    fn body(&self) -> &Body<'a> {
        &self.body
    }
}
impl<'a> IsMutItem<'a> for Item<'a> {
    fn mut_body(&mut self) -> &mut Body<'a> {
        &mut self.body
    }
}

impl From<&str> for AttrValue {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl Add<&AttrValue> for AttrValue {
    type Output = Self;

    fn add(self, other: &AttrValue) -> Self::Output {
        match other {
            Self::String(other_string) => match self {
                Self::String(string) => Self::String(string + other_string),
                _ => self,
            },
            Self::Bool(other_bool) => match self {
                Self::Bool(_) => Self::Bool(other_bool.clone()),
                _ => self,
            },
        }
    }
}

impl<'a> From<&str> for ContentPart<'a> {
    fn from(text: &str) -> Self {
        ContentPart::Text(text.into())
    }
}

impl<'a, P: Into<ContentPart<'a>>> Add<P> for Content<'a> {
    type Output = Self;

    fn add(self, part: P) -> Self::Output {
        Content(self.0.into_iter().chain([part.into()]).collect())
    }
}

impl<'a> Add<&ContentPart<'a>> for Content<'a> {
    type Output = Self;

    fn add(self, part: &ContentPart<'a>) -> Self::Output {
        self + part.clone()
    }
}

impl<'a, P: Into<ContentPart<'a>>> From<P> for Content<'a> {
    fn from(part: P) -> Self {
        Content(vec![part.into()])
    }
}

impl<'a> Add<&Content<'a>> for Content<'a> {
    type Output = Self;

    fn add(self, other: &Content<'a>) -> Self::Output {
        Content(self.0.into_iter().chain(other.0.iter().cloned()).collect())
    }
}

impl AttrValue {
    fn format_string(string: &str) -> String {
        format!("\"{string}\"")
    }
    fn format_bool(bool: &bool) -> String {
        bool.to_string()
    }
}

impl<'a> fmt::Display for AttrValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::String(string) => Self::format_string(string),
                Self::Bool(bool) => Self::format_bool(bool),
            }
        )
    }
}

impl<'a> fmt::Display for ContentPart<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Text(text) => Content::format_text(text),
                Self::Item(item) => item.to_string(),
            }
        )
    }
}

impl<'a> Content<'a> {
    fn format_text(text: &str) -> String {
        format!("\"{text}\"")
    }
    fn format_item(tag: &str, attrs: &str, content: &str) -> String {
        format!(
            "<{tag}{attrs}>{}",
            if !EMPTY_TAGS.contains(&tag) {
                format!("{content}</{tag}>")
            } else {
                "".into()
            }
        )
    }

    fn slot_content(&self, slot: &str) -> Option<String> {
        self.0
            .iter()
            .filter(|part| match part {
                ContentPart::Text(_) => slot == DEFAULT_SLOT,
                ContentPart::Item(item) => item.attr("slot").map_or_else(
                    || slot == DEFAULT_SLOT,
                    |s| match s {
                        AttrValue::String(s) => s == slot,
                        _ => false,
                    },
                ),
            })
            .map(|p| p.to_string())
            .reduce(|p1, p2| p1 + &p2)
    }

    fn to_string_by(&self, template: &Self) -> String {
        template
            .0
            .iter()
            .fold(String::new(), |content, template_part| {
                content
                    + &match template_part {
                        ContentPart::Text(template_text) => Self::format_text(template_text),
                        ContentPart::Item(template_item) => Self::format_item(
                            template_item.tag(),
                            &template_item.attrs_string(),
                            &if let Some(slot) = template_item.wrapper().slot() {
                                self.slot_content(slot).unwrap_or_else(|| {
                                    template_item
                                        .content()
                                        .map_or_else(|| "".into(), |c| c.to_string())
                                })
                            } else {
                                template_item
                                    .content()
                                    .map_or_else(|| "".into(), |t| self.to_string_by(t))
                            },
                        ),
                    }
            })
    }
}

impl<'a> fmt::Display for Content<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0.iter().fold(String::new(), |content, part| content
                + &match part {
                    ContentPart::Text(text) => Self::format_text(text),
                    ContentPart::Item(item) => item.to_string(),
                })
        )
    }
}

impl<'a> fmt::Display for Item<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            Content::format_item(
                self.tag(),
                &self.attrs_string(),
                &self.content().map_or_else(
                    || "".into(),
                    |content| self.wrapper().content().map_or_else(
                        || content.to_string(),
                        |template| content.to_string_by(template)
                    )
                ),
            )
        )
    }
}
