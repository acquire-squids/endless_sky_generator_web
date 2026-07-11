crate::macros::wasm_newtype! {
    in main =>
    #[derive(Debug)]
    pub ChaosConfig;
    pub seed: u64,
}

pub mod from_file {
    use crate::{
        config::{self, Value},
        generators::chaos::config::ChaosConfig,
    };

    #[allow(unreachable_patterns)]
    #[must_use]
    pub fn parse(source: &str) -> Option<ChaosConfig> {
        config::parse_config!(
            source => ChaosConfig;
            seed => { int of u64 => seed }
        )
    }
}

pub mod page {
    use crate::{
        generators::chaos::config,
        html::{self, HtmlElement},
    };

    const DEFAULT_CONFIG_FILE: &str = include_str!("../../../config/chaos/default.txt");

    #[must_use]
    pub fn chaos() -> HtmlElement {
        HtmlElement::new("form")
                .with_name("chaos-form")
                .with_id("chaos-form")
                .novalidate()
                .with_element(
                    HtmlElement::new("h2")
                        .with_element(
                            html::page::anchor("Chaos", "Chaos")
                        )
                )
                .with_element(
                    HtmlElement::new("p")
                        .with_text("This plugin shuffles the sprites, thumbnails, and names of every ship and outfit.<br/>")
                        .with_text("Everything will play the same, mostly, but the hitboxes for ships will be different and you won't know what anything is at a glance")
                )
                .with_element(
                    chaos_fieldset()
                )
                .with_element(
                    HtmlElement::new("button")
                        .with_id("chaos-output")
                        .with_attribute("type", "submit")
                        .with_text("Generate and download")
                )
    }

    fn chaos_fieldset() -> HtmlElement {
        let settings = config::from_file::parse(DEFAULT_CONFIG_FILE);
        let settings = settings.as_ref();

        HtmlElement::new("fieldset")
            .with_element(HtmlElement::new("legend").with_text("Chaos Settings:"))
            .with_element(html::page::labeled("chaos-seed", "", "seed:", {
                let input = HtmlElement::new("input")
                    .with_attribute("type", "number")
                    .required();

                if let Some(settings) = settings {
                    input.with_attribute("value", *settings.seed())
                } else {
                    input
                }
            }))
    }
}
