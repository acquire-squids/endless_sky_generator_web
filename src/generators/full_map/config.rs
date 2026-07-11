pub mod page {
    use crate::html::{self, HtmlElement};

    #[must_use]
    pub fn full_map() -> HtmlElement {
        HtmlElement::new("div")
            .with_element(
                HtmlElement::new("h2")
                    .with_element(
                        html::page::anchor("Full_Map", "Full Map")
                    )
            )
            .with_element(
                HtmlElement::new("p")
                    .with_text("A plugin that reveals every system and planet via a job available in any job board.<br/><br/>")
                    .with_text("This works by reading all `system` root nodes.<br/>")
                    .with_text("If a system is hidden or shrouded, it may not remain revealed after takeoff.")
            )
            .with_element(
                HtmlElement::new("button")
                    .with_id("full-map-output")
                    .with_attribute("type", "submit")
                    .with_text("Generate and download")
            )
    }
}
