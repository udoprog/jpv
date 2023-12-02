use std::borrow::Cow;

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

#[derive(Default, Debug)]
pub(crate) struct Query {
    pub(crate) q: String,
    pub(crate) translation: Option<String>,
    pub(crate) a: Vec<String>,
    pub(crate) i: usize,
    pub(crate) mode: Mode,
    pub(crate) capture_clipboard: bool,
    pub(crate) embed: bool,
    pub(crate) tab: Tab,
}

impl Query {
    pub(crate) fn deserialize(raw: Vec<(String, String)>) -> (Self, Option<usize>) {
        let mut this = Self::default();
        let mut analyze_at = None;

        for (key, value) in raw {
            match key.as_str() {
                "q" => {
                    this.q = value;
                }
                "t" => {
                    this.translation = Some(value);
                }
                "a" => {
                    this.a.push(value);
                }
                "i" => {
                    if let Ok(i) = value.parse() {
                        this.i = i;
                    }
                }
                "mode" => {
                    this.mode = match value.as_str() {
                        "hiragana" => Mode::Hiragana,
                        "katakana" => Mode::Katakana,
                        _ => Mode::Unfiltered,
                    };
                }
                "cb" => {
                    this.capture_clipboard = value == "yes";
                }
                "embed" => {
                    this.embed = value == "yes";
                }
                "tab" => {
                    this.tab = match value.as_str() {
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

        (this, analyze_at)
    }

    pub(crate) fn serialize(&self) -> Vec<(&'static str, Cow<'_, str>)> {
        let mut out = Vec::new();

        if !self.q.is_empty() {
            out.push(("q", Cow::Borrowed(self.q.as_str())));
        }

        if let Some(t) = &self.translation {
            out.push(("t", Cow::Borrowed(t)));
        }

        for a in &self.a {
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

        if self.embed {
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
