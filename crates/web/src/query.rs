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
    pub(crate) a: Rc<[String]>,
    pub(crate) i: usize,
    pub(crate) mode: Mode,
    pub(crate) capture_clipboard: bool,
    pub(crate) embed: bool,
    pub(crate) tab: Tab,
}

impl Query {
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
        let mut text = String::new();
        let mut translation = None;
        let mut a = Vec::new();
        let mut index = 0;
        let mut mode = Mode::default();
        let mut capture_clipboard = false;
        let mut embed = false;
        let mut tab = Tab::default();

        for (key, value) in raw {
            match key.as_str() {
                "q" => {
                    text = value;
                }
                "t" => {
                    translation = Some(value);
                }
                "a" => {
                    a.push(value);
                }
                "i" => {
                    if let Ok(i) = value.parse() {
                        index = i;
                    }
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
                "analyzeAt" => {
                    if let Ok(i) = value.parse() {
                        analyze_at = Some(i);
                    }
                }
                _ => {}
            }
        }

        let this = Self {
            text: text.into(),
            translation,
            a: a.into(),
            i: index,
            mode,
            capture_clipboard,
            embed,
            tab,
        };

        (this, analyze_at)
    }

    pub(crate) fn serialize(&self, no_embed: bool) -> Vec<(&'static str, Cow<'_, str>)> {
        let mut out = Vec::new();

        if !self.text.is_empty() {
            out.push(("q", Cow::Borrowed(self.text.as_ref())));
        }

        if let Some(t) = &self.translation {
            out.push(("t", Cow::Borrowed(t)));
        }

        for a in self.a.iter() {
            out.push(("a", Cow::Borrowed(a.as_str())));
        }

        if self.i != 0 {
            out.push(("i", Cow::Owned(self.i.to_string())));
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

        out
    }
}
