use std::fmt;

pub struct HtmlPage {
    html: Html,
}

pub enum Html {
    Element(HtmlElement),
    Text(String),
}

pub struct HtmlElement {
    kind: String,
    attributes: Vec<HtmlAttribute>,
    children: Vec<Html>,
}

enum HtmlAttribute {
    KeyValuePair { key: String, value: String },
    KeyOnly(String),
}

pub trait AttributeValue: Clone + fmt::Display {
    const QUOTED: bool = false;

    fn to_string(&self) -> String {
        if Self::QUOTED {
            format!("\"{self}\"")
        } else {
            format!("{self}")
        }
    }
}

impl AttributeValue for u8 {}
impl AttributeValue for u16 {}
impl AttributeValue for u32 {}
impl AttributeValue for u64 {}
impl AttributeValue for u128 {}

impl AttributeValue for i8 {}
impl AttributeValue for i16 {}
impl AttributeValue for i32 {}
impl AttributeValue for i64 {}
impl AttributeValue for i128 {}

impl AttributeValue for f32 {}
impl AttributeValue for f64 {}

impl AttributeValue for &str {
    const QUOTED: bool = true;
}

impl AttributeValue for String {
    const QUOTED: bool = true;
}

impl fmt::Display for HtmlPage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<!doctype html>\n{}", self.html)
    }
}

impl fmt::Display for Html {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(text) => write!(f, "{text}"),
            Self::Element(element) => write!(f, "{element}"),
        }
    }
}

impl fmt::Display for HtmlElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}", self.kind)?;

        for attribute in &self.attributes {
            write!(f, " {attribute}")?;
        }

        write!(f, ">")?;

        for child in &self.children {
            write!(f, "{child}")?;
        }

        write!(f, "</{}>", self.kind)
    }
}

impl fmt::Display for HtmlAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyValuePair { key, value } => write!(f, "{key}={value}"),
            Self::KeyOnly(key) => write!(f, "{key}"),
        }
    }
}

impl HtmlElement {
    pub fn new<Kind>(kind: Kind) -> Self
    where
        Kind: AsRef<str>,
    {
        Self {
            kind: kind.as_ref().to_string(),
            attributes: vec![],
            children: vec![],
        }
    }

    #[must_use]
    pub fn with_child(mut self, child: Html) -> Self {
        self.children.push(child);

        self
    }

    #[must_use]
    pub fn with_text<Text>(self, text: Text) -> Self
    where
        Text: AsRef<str>,
    {
        self.with_child(Html::Text(text.as_ref().to_string()))
    }

    #[must_use]
    pub fn with_element(self, element: Self) -> Self {
        self.with_child(Html::Element(element))
    }

    #[must_use]
    pub fn with_div(self, element: Self) -> Self {
        self.with_element(Self::new("div").with_element(element))
    }

    #[must_use]
    pub fn with_attributes<Key, Value>(self, attributes: Vec<(Key, Value)>) -> Self
    where
        Key: AsRef<str>,
        Value: AttributeValue,
    {
        let mut element = self;

        for (key, value) in attributes {
            element = element.with_attribute(key, value);
        }

        element
    }

    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn with_attribute<Key, Value>(mut self, key: Key, value: Value) -> Self
    where
        Key: AsRef<str>,
        Value: AttributeValue,
    {
        if let Some(HtmlAttribute::KeyValuePair { value: existing_value, .. }) = self.attributes.iter_mut().find(|attribute| {
            matches!(attribute, HtmlAttribute::KeyValuePair { key: existing_key, .. } if existing_key == key.as_ref())
        }) {
            *existing_value = value.to_string();
        } else {
            self.attributes.push(HtmlAttribute::KeyValuePair {
                key: key.as_ref().to_string(),
                value: value.to_string(),
            });
        }

        self
    }

    #[must_use]
    pub fn required(mut self) -> Self {
        let toggle_attribute = "required";

        if !self.attributes.iter().any(
            |attribute| matches!(attribute, HtmlAttribute::KeyOnly(key) if key == toggle_attribute),
        ) {
            self.attributes
                .push(HtmlAttribute::KeyOnly(toggle_attribute.to_string()));
        }

        self
    }

    #[must_use]
    pub fn checked(mut self) -> Self {
        let toggle_attribute = "checked";

        if !self.attributes.iter().any(
            |attribute| matches!(attribute, HtmlAttribute::KeyOnly(key) if key == toggle_attribute),
        ) {
            self.attributes
                .push(HtmlAttribute::KeyOnly(toggle_attribute.to_string()));
        }

        self
    }

    #[must_use]
    pub fn webkitdirectory(mut self) -> Self {
        let toggle_attribute = "webkitdirectory";

        if !self.attributes.iter().any(
            |attribute| matches!(attribute, HtmlAttribute::KeyOnly(key) if key == toggle_attribute),
        ) {
            self.attributes
                .push(HtmlAttribute::KeyOnly(toggle_attribute.to_string()));
        }

        self
    }

    #[must_use]
    pub fn open(mut self) -> Self {
        let toggle_attribute = "open";

        if !self.attributes.iter().any(
            |attribute| matches!(attribute, HtmlAttribute::KeyOnly(key) if key == toggle_attribute),
        ) {
            self.attributes
                .push(HtmlAttribute::KeyOnly(toggle_attribute.to_string()));
        }

        self
    }

    #[must_use]
    pub fn novalidate(mut self) -> Self {
        let toggle_attribute = "novalidate";

        if !self.attributes.iter().any(
            |attribute| matches!(attribute, HtmlAttribute::KeyOnly(key) if key == toggle_attribute),
        ) {
            self.attributes
                .push(HtmlAttribute::KeyOnly(toggle_attribute.to_string()));
        }

        self
    }

    #[must_use]
    pub fn with_class<Value>(self, class_name: Value) -> Self
    where
        Value: AttributeValue,
    {
        self.with_attribute("class", class_name)
    }

    #[must_use]
    pub fn with_name<Value>(self, name: Value) -> Self
    where
        Value: AttributeValue,
    {
        self.with_attribute("name", name)
    }

    #[must_use]
    pub fn with_id<Value>(self, id: Value) -> Self
    where
        Value: AttributeValue,
    {
        self.with_attribute("id", id)
    }
}

#[must_use]
pub fn page_contents() -> String {
    page::contents().to_string()
}

// TODO: make this less verbose
// TODO: maybe find a way to make this rely less on strings (Rust has a solid type system, use it)
pub mod page {
    use crate::generators::{
        chaos::config::page as chaos_form, full_map::config::page as full_map_form,
        random_galaxy::config::page as random_galaxy_form,
        system_shuffler::config::page as system_shuffler_form,
    };

    use super::{AttributeValue, Html, HtmlElement, HtmlPage};

    const PAGE_NAME: &str = "Squiddy's Endless Sky Plugins Generator";

    #[must_use]
    pub fn contents() -> HtmlPage {
        HtmlPage {
            html: Html::Element(
                HtmlElement::new("html")
                    .with_attributes(vec![("lang", "en-US")])
                    .with_element(head())
                    .with_element(body()),
            ),
        }
    }

    fn head() -> HtmlElement {
        HtmlElement::new("head")
            .with_element(HtmlElement::new("meta").with_attribute("charset", "utf-8"))
            .with_element(HtmlElement::new("meta").with_attributes(vec![
                ("name", "viewport"),
                ("content", "width=device-width"),
            ]))
            .with_element(HtmlElement::new("title").with_text(PAGE_NAME))
    }

    fn header() -> HtmlElement {
        HtmlElement::new("header").with_element(HtmlElement::new("h1").with_text(PAGE_NAME))
    }

    #[must_use]
    pub fn anchor<Tag, Anchor>(tag: Tag, text: Anchor) -> HtmlElement
    where
        Tag: AttributeValue,
        Anchor: AsRef<str>,
    {
        HtmlElement::new("a").with_id(tag).with_text(text)
    }

    fn goto<Tag, Anchor>(tag: Tag, text: Anchor) -> HtmlElement
    where
        Tag: AsRef<str>,
        Anchor: AsRef<str>,
    {
        HtmlElement::new("a")
            .with_attribute("href", format!("#{}", tag.as_ref()))
            .with_text(text)
    }

    #[must_use]
    pub fn weight<WeightIsFor>(
        weight_is_for: WeightIsFor,
        default_weight: Option<u32>,
    ) -> HtmlElement
    where
        WeightIsFor: AsRef<str>,
    {
        labeled(
            weight_is_for.as_ref(),
            "weight:",
            HtmlElement::new("input")
                .with_attribute("type", "number")
                .required()
                .with_attributes(vec![("value", default_weight.unwrap_or(1)), ("min", 1u32)]),
        )
    }

    #[must_use]
    pub fn labeled<LabelIsFor, Label>(
        label_is_for: LabelIsFor,
        label: Label,
        element: HtmlElement,
    ) -> HtmlElement
    where
        LabelIsFor: AsRef<str>,
        Label: AsRef<str>,
    {
        HtmlElement::new("div")
            .with_element(
                HtmlElement::new("label")
                    .with_attribute("for", label_is_for.as_ref())
                    .with_text(format!("{} ", label.as_ref())),
            )
            .with_element(
                element
                    .with_class(label_is_for.as_ref())
                    .with_name(label_is_for.as_ref())
                    .with_id(label_is_for.as_ref()),
            )
    }

    #[must_use]
    pub fn labeled_range<LabelIsFor, Label, Value>(
        label_is_for: LabelIsFor,
        label: Label,
        value: Value,
        (min, max): (Value, Value),
        any_step: bool,
    ) -> HtmlElement
    where
        LabelIsFor: AsRef<str>,
        Label: AsRef<str>,
        Value: AttributeValue,
    {
        HtmlElement::new("div")
            .with_element(
                HtmlElement::new("label")
                    .with_attribute("for", label_is_for.as_ref())
                    .with_text(format!("{} ", label.as_ref())),
            )
            .with_element(
                HtmlElement::new("input")
                    .with_class(format!("paired-range {}", label_is_for.as_ref()))
                    .with_name(label_is_for.as_ref())
                    .with_id(label_is_for.as_ref())
                    .with_attribute("type", "range")
                    .with_attributes(vec![
                        ("value", value.clone()),
                        ("min", min.clone()),
                        ("max", max.clone()),
                    ])
                    .required(),
            )
            .with_element({
                let input = HtmlElement::new("input")
                    .with_class(format!("paired-range-output {}", label_is_for.as_ref()))
                    .with_name(format!("{}-output", label_is_for.as_ref()))
                    .with_id(format!("{}-output", label_is_for.as_ref()))
                    .with_attribute("type", "number")
                    .with_attributes(vec![("value", value), ("min", min), ("max", max)])
                    .required();

                if any_step {
                    input.with_attribute("step", "any")
                } else {
                    input.with_attribute("step", 1)
                }
            })
    }

    #[must_use]
    pub fn labeled_min_max<LabelIsFor, Label, Value>(
        (label_is_for_min, label_is_for_max): (LabelIsFor, LabelIsFor),
        label: Label,
        (default_min, default_max): (Value, Value),
        (min, max): (Value, Value),
        any_step: bool,
    ) -> HtmlElement
    where
        LabelIsFor: AsRef<str>,
        Label: AsRef<str>,
        Value: AttributeValue,
    {
        HtmlElement::new("label")
            .with_text(format!("{} ", label.as_ref()))
            .with_element({
                let input = HtmlElement::new("input")
                    .with_class(label_is_for_min.as_ref())
                    .with_name(label_is_for_min.as_ref())
                    .with_id(label_is_for_min.as_ref())
                    .with_attribute("type", "number")
                    .with_attributes(vec![
                        ("value", default_min),
                        ("min", min.clone()),
                        ("max", max.clone()),
                    ])
                    .required();

                if any_step {
                    input.with_attribute("step", "any")
                } else {
                    input.with_attribute("step", 1)
                }
            })
            .with_element({
                let input = HtmlElement::new("input")
                    .with_class(label_is_for_max.as_ref())
                    .with_name(label_is_for_max.as_ref())
                    .with_id(label_is_for_max.as_ref())
                    .with_attribute("type", "number")
                    .with_attributes(vec![("value", default_max), ("min", min), ("max", max)])
                    .required();

                if any_step {
                    input.with_attribute("step", "any")
                } else {
                    input.with_attribute("step", 1)
                }
            })
    }

    #[must_use]
    pub fn fieldset_group<Legend, Id, ItemLabel, RemoveLabel, NewLabel>(
        legend: Legend,
        id: Id,
        item_label: ItemLabel,
        (remove_text, new_text): (RemoveLabel, NewLabel),
        fields: Vec<HtmlElement>,
    ) -> HtmlElement
    where
        Legend: AsRef<str>,
        Id: AsRef<str>,
        ItemLabel: AsRef<str>,
        RemoveLabel: AsRef<str>,
        NewLabel: AsRef<str>,
    {
        HtmlElement::new("details")
            .open()
            .with_element(HtmlElement::new("summary").with_text(format!("{} ", legend.as_ref())))
            .with_element(
                HtmlElement::new("fieldset")
                    .with_element({
                        let mut fieldset = HtmlElement::new("fieldset").with_element(
                            HtmlElement::new("button")
                                .with_class("click-to-remove")
                                .with_attribute("type", "button")
                                .with_text(remove_text.as_ref())
                                .with_attribute("data-commandfor", id.as_ref()),
                        );

                        for field in fields {
                            fieldset = fieldset.with_element(field);
                        }

                        HtmlElement::new("details")
                            .with_class(format!("can-be-created can-be-removed {}", id.as_ref()))
                            .with_name(id.as_ref())
                            .with_id(id.as_ref())
                            .with_element(
                                HtmlElement::new("summary").with_text(item_label.as_ref()),
                            )
                            .with_element(fieldset)
                    })
                    .with_element(
                        HtmlElement::new("button")
                            .with_class("click-to-create")
                            .with_attribute("type", "button")
                            .with_text(new_text.as_ref())
                            .with_attribute("data-commandfor", id.as_ref()),
                    ),
            )
    }

    fn body() -> HtmlElement {
        HtmlElement::new("body")
            .with_element(header())
            .with_element(instructions())
            .with_element(full_map_form::full_map())
            .with_element(system_shuffler_form::system_shuffler())
            .with_element(chaos_form::chaos())
            .with_element(random_galaxy_form::random_galaxy())
            .with_element(
                HtmlElement::new("script")
                    .with_attribute("type", "module")
                    .with_attribute("src", "./index.js"),
            )
    }

    fn instructions() -> HtmlElement {
        HtmlElement::new("div")
            .with_element(
                HtmlElement::new("h2")
                    .with_element(
                        anchor("READ_ME", "READ ME")
                    )
            )
            .with_div(
                HtmlElement::new("p")
                    .with_text("<b>The generators will include the base game's data for this version by default:</b>")
            )
            .with_div(
                HtmlElement::new("object")
                    .with_attributes(vec![
                        ("type", "text/plain"),
                        ("data", "stable_version.txt"),
                    ])
                    .with_attributes(vec![
                        ("width", 96u32),
                        ("height", 24u32),
                    ])
                    .with_text("!!UNKNOWN!!")
            )
            .with_div(
                HtmlElement::new("p")
                    .with_text("If you don't want this, uncheck the \"Include data from stable release\" checkbox below.<br/><br/>")
                    .with_text("You can also try to include your own data files, if you'd like!  Use the \"Upload your own data folder\" button.<br/>")
                    .with_text("If you wish to clear the data you've \"uploaded\", there is a button right below the upload button.<br/><br/>")
                    .with_text("These settings, and the uploaded files, <b>are not stored (to my knowledge) and must be repeated if you come back!</b><br/>")
            )
            .with_element(
                labeled(
                    "include-defaults",
                    "Include data from stable release:",
                    HtmlElement::new("input")
                        .with_attribute("type", "checkbox")
                        .checked()
                )
            )
            .with_element(
                labeled(
                    "input",
                    "Upload your own data folder:",
                    HtmlElement::new("input")
                        .with_attribute("type", "file")
                        .webkitdirectory()
                )
            )
            .with_div(
                HtmlElement::new("button")
                    .with_id("clear-uploads")
                    .with_attribute("type", "button")
                    .with_text("Clear uploaded data")
            )
            .with_element(
                table_of_contents()
            )
    }

    fn table_of_contents() -> HtmlElement {
        HtmlElement::new("div")
            .with_element(HtmlElement::new("h3").with_text("Table of contents"))
            .with_element(
                HtmlElement::new("ul")
                    .with_element(HtmlElement::new("li").with_element(goto("READ_ME", "READ ME")))
                    .with_element(HtmlElement::new("li").with_element(goto("Full_Map", "Full Map")))
                    .with_element(
                        HtmlElement::new("li")
                            .with_element(goto("System_Shuffler", "System Shuffler")),
                    )
                    .with_element(HtmlElement::new("li").with_element(goto("Chaos", "Chaos")))
                    .with_element(
                        HtmlElement::new("li").with_element(goto("Random_Galaxy", "Random Galaxy")),
                    ),
            )
    }
}
