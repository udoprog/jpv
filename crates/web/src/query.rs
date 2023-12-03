use std::{borrow::Cow, rc::Rc};

use web_sys::{window, Url};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    #[default]
    Unfiltered,
    Hiragana,
    Katakana,
}

/// The current tab.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Phrases,
    Names,
    Kanji,
    Settings,
}

#[derive(Debug)]
pub(crate) struct Query {
    pub(crate) text: Rc<str>,
    pub(crate) translation: Option<String>,
    pub(crate) analyze_at: Option<usize>,
    pub(crate) index: usize,
    pub(crate) mode: Mode,
    pub(crate) capture_clipboard: bool,
    pub(crate) embed: bool,
    pub(crate) tab: Tab,
}

impl Query {
    /// Update query in the most common way.
    pub(crate) fn set(&mut self, text: Rc<str>, translation: Option<String>) {
        self.text = text;
        self.translation = translation;
        self.analyze_at = None;
        self.index = 0;
    }

    pub(crate) fn to_href(&self, no_embed: bool) -> Option<String> {
        let href = window()?.location().href().ok()?;
        let query = self.serialize(no_embed);
        let query = serde_urlencoded::to_string(&query).ok()?;
        let url = Url::new_with_base("/", &href).ok()?;
        url.set_search(&query);
        Some(url.href())
    }

    pub(crate) fn deserialize(raw: Vec<(String, String)>) -> (Self, Option<usize>) {
        let mut analyze_at = None;
        let mut analyze_at_char = None;
        let mut text = String::new();
        let mut translation = None;
        let mut mode = Mode::default();
        let mut capture_clipboard = false;
        let mut embed = false;
        let mut tab = Tab::default();
        let mut index = 0;

        for (key, value) in raw {
            match key.as_str() {
                "q" => {
                    text = value;
                }
                "t" => {
                    translation = Some(value);
                }
                "mode" => {
                    mode = match value.as_str() {
                        "hiragana" => Mode::Hiragana,
                        "katakana" => Mode::Katakana,
                        _ => Mode::Unfiltered,
                    };
                }
                "cb" => {
                    capture_clipboard = value == "yes";
                }
                "embed" => {
                    embed = value == "yes";
                }
                "tab" => {
                    tab = match value.as_str() {
                        "phrases" => Tab::Phrases,
                        "names" => Tab::Names,
                        "kanji" => Tab::Kanji,
                        "settings" => Tab::Settings,
                        _ => Tab::default(),
                    };
                }
                "at" => {
                    if let Ok(i) = value.parse() {
                        analyze_at = Some(i);
                    }
                }
                "analyze_at_char" => {
                    if let Ok(i) = value.parse() {
                        analyze_at_char = Some(i);
                    }
                }
                "index" => {
                    if let Ok(i) = value.parse() {
                        index = i;
                    }
                }
                _ => {}
            }
        }

        let this = Self {
            text: text.into(),
            translation,
            mode,
            capture_clipboard,
            embed,
            tab,
            analyze_at,
            index,
        };

        (this, analyze_at_char)
    }

    pub(crate) fn serialize(&self, no_embed: bool) -> Vec<(&'static str, Cow<'_, str>)> {
        let mut out = Vec::new();

        if !self.text.is_empty() {
            out.push(("q", Cow::Borrowed(self.text.as_ref())));
        }

        if let Some(t) = &self.translation {
            out.push(("t", Cow::Borrowed(t)));
        }

        match self.mode {
            Mode::Unfiltered => {}
            Mode::Hiragana => {
                out.push(("mode", Cow::Borrowed("hiragana")));
            }
            Mode::Katakana => {
                out.push(("mode", Cow::Borrowed("katakana")));
            }
        }

        if self.capture_clipboard {
            out.push(("cb", Cow::Borrowed("yes")));
        }

        if !no_embed && self.embed {
            out.push(("embed", Cow::Borrowed("yes")));
        }

        if let Some(analyze_at) = self.analyze_at {
            out.push(("at", Cow::Owned(analyze_at.to_string())));
        }

        match self.tab {
            Tab::Phrases => {}
            Tab::Names => {
                out.push(("tab", Cow::Borrowed("names")));
            }
            Tab::Kanji => {
                out.push(("tab", Cow::Borrowed("kanji")));
            }
            Tab::Settings => {
                out.push(("tab", Cow::Borrowed("settings")));
            }
        }

        if self.index > 0 {
            out.push(("index", Cow::Owned(self.index.to_string())));
        }

        out
    }
}
