crate::macros::wasm_newtype! {
    in main =>
    #[derive(Debug)]
    pub SystemShufflerConfig ;
    seed: u64,
    max_presets: u8,
    shuffle_chance: u8,
    fixed_shuffle_days: u8,
    shuffle_once_on_install: bool,
}

pub mod from_file {
    use crate::{
        config::{self, Value},
        generators::system_shuffler::config::SystemShufflerConfig,
    };

    #[allow(unreachable_patterns)]
    #[must_use]
    pub fn parse(source: &str) -> Option<SystemShufflerConfig> {
        config::parse_config!(
            source => SystemShufflerConfig;
            seed => { int of u64 => seed }
            max_presets => { int of u8 => max_presets }
            shuffle_chance => { int of u8 where shuffle_chance <= 100 => shuffle_chance }
            fixed_shuffle_days => { int of u8 => fixed_shuffle_days }
            shuffle_once_on_install => { bool => *shuffle_once_on_install }
        )
    }
}

pub mod page {
    use crate::{
        generators::system_shuffler::config,
        html::{self, HtmlElement},
    };

    const DEFAULT_CONFIG_FILE: &str = include_str!("../../../config/system_shuffler/default.txt");

    #[must_use]
    pub fn system_shuffler() -> HtmlElement {
        HtmlElement::new("form")
            .with_name("system-shuffler-form")
            .with_id("system-shuffler-form")
            .novalidate()
            .with_element(
                HtmlElement::new("h2")
                    .with_element(
                        html::page::anchor("System_Shuffler", "System Shuffler")
                    )
            )
            .with_element(
                HtmlElement::new("p")
                    .with_text("A plugin that comes with preset randomizations of every system in the universe.<br/><br/>")
                    .with_text("It will swap system positions but keep the shape and traversability of the universe the same.<br/>")
                    .with_text("Using the inputs below, you can configure the plugin to shuffle the universe into a random preset on certain conditions:")
                    .with_element(
                        HtmlElement::new("ul")
                            .with_element(
                                HtmlElement::new("li")
                                    .with_text("Once, immediately, upon installation")
                            )
                            .with_element(
                                HtmlElement::new("li")
                                    .with_text("With a percentage chance every time you land")
                            )
                            .with_element(
                                HtmlElement::new("li")
                                    .with_text("Every N days, with the shuffle happening after a greater amount of time if you have not landed for N days")
                            )
                    )
                    .with_text("Additionally, you can request a shuffle or restore the universe at any point through the job board.<br/><br/>")
                    .with_text("<b>Be wary of repeated shuffling!</b><br/>")
                    .with_text("If you play on a version <b>before v0.11.0's unstable release</b>, event definitions are fully copied into your save file and <b>your save file has potential to explode in size!</b><br/><br/>")
                    .with_text("Don't forget to <b>back up your saves before use!</b>")
            )
            .with_element(
                system_shuffler_fieldset()
            )
            .with_element(
                HtmlElement::new("button")
                    .with_id("system-shuffler-output")
                    .with_attribute("type", "submit")
                    .with_text("Generate and download")
            )
    }

    fn system_shuffler_fieldset() -> HtmlElement {
        let settings = config::from_file::parse(DEFAULT_CONFIG_FILE);
        let settings = settings.as_ref();

        HtmlElement::new("fieldset")
            .with_element(HtmlElement::new("legend").with_text("System Shuffler Settings:"))
            .with_element(html::page::labeled("system-shuffler-seed", "", "seed:", {
                let input = HtmlElement::new("input")
                    .with_attribute("type", "number")
                    .required();

                if let Some(settings) = settings {
                    input.with_attribute("value", *settings.seed())
                } else {
                    input
                }
            }))
            .with_element(html::page::labeled(
                "system-shuffler-max-presets",
                "",
                "max presets:",
                {
                    let input = HtmlElement::new("input")
                        .with_attribute("type", "number")
                        .required()
                        .with_attribute("min", 1u32)
                        .with_attribute("max", 255u32);

                    if let Some(settings) = settings {
                        input.with_attribute("value", *settings.max_presets())
                    } else {
                        input
                    }
                },
            ))
            .with_element(html::page::labeled(
                "system-shuffler-shuffle-once-on-install",
                "",
                "shuffle once upon installation:",
                {
                    let input = HtmlElement::new("input").with_attribute("type", "checkbox");

                    if let Some(settings) = settings
                        && *settings.shuffle_once_on_install()
                    {
                        input.checked()
                    } else {
                        input
                    }
                },
            ))
            .with_element(html::page::labeled_range(
                "system-shuffler-shuffle-chance",
                "",
                "chance to shuffle every time you land:",
                settings.map_or(0u8, |settings| *settings.shuffle_chance()),
                (0u8, 100u8),
                false,
            ))
            .with_element(html::page::labeled_range(
                "system-shuffler-fixed-shuffle-days",
                "",
                "shuffle every N days (0 to disable):",
                settings.map_or(0u8, |settings| *settings.fixed_shuffle_days()),
                (0u8, 255u8),
                false,
            ))
    }
}
