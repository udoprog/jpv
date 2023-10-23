use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

use lib::database::EntryResultKey;
use lib::jmdict::{EntryKey, OwnedEntry};
use lib::romaji;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::{prelude::*, AnyRoute};

use crate::fetch::FetchError;
use crate::{components as c, fetch};

pub(crate) enum Msg {
    Mode(Mode),
    Change(String),
    ForceChange(String, Option<String>),
    Analyze(usize),
    AnalyzeCycle,
    HistoryChanged(Location),
    SearchResponse(fetch::SearchResponse),
    AnalyzeResponse(fetch::AnalyzeResponse),
    Error(FetchError),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    #[default]
    Unfiltered,
    Hiragana,
    Katakana,
}

#[derive(Default, Debug)]
struct Query {
    q: String,
    translation: Option<String>,
    a: Vec<String>,
    i: usize,
    mode: Mode,
}

impl Query {
    fn deserialize(raw: Vec<(String, String)>) -> Self {
        let mut this = Self::default();

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
                _ => {}
            }
        }

        this
    }

    fn serialize(&self) -> Vec<(&'static str, Cow<'_, str>)> {
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

        out
    }
}

#[derive(Default)]
pub(crate) struct Prompt {
    query: Query,
    entries: Vec<(EntryResultKey, OwnedEntry)>,
    _handle: Option<LocationHandle>,
}

impl Prompt {
    fn refresh(&mut self, ctx: &Context<Self>, input: &str) {
        if let Some(db) = &*ctx.props().db {
            let input = input.to_lowercase();

            let entries = match db.search(&input) {
                Ok(entries) => entries,
                Err(error) => {
                    log::error!("Search failed: {error}");
                    return;
                }
            };

            self.entries = entries
                .into_iter()
                .map(|(key, e)| (key, borrowme::to_owned(e)))
                .collect();

            self.entries.sort_by(|(a, _), (b, _)| a.key.cmp(&b.key));
        } else {
            let input = input.to_lowercase();

            ctx.link().send_future(async move {
                match fetch::search(&input).await {
                    Ok(entries) => Msg::SearchResponse(entries),
                    Err(error) => Msg::Error(error),
                }
            });
        }
    }

    fn analyze(&mut self, ctx: &Context<Self>, start: usize) -> Option<BTreeMap<EntryKey, String>> {
        let Some(db) = &*ctx.props().db else {
            let input = self.query.q.clone();

            ctx.link().send_future(async move {
                match fetch::analyze(&input, start).await {
                    Ok(entries) => Msg::AnalyzeResponse(entries),
                    Err(error) => Msg::Error(error),
                }
            });

            return None;
        };

        Some(db.analyze(&self.query.q, start))
    }

    fn save_query(&mut self, ctx: &Context<Prompt>, push: bool) {
        if let (Some(location), Some(navigator)) = (ctx.link().location(), ctx.link().navigator()) {
            let path = location.path();
            let path = AnyRoute::new(path);

            let query = self.query.serialize();

            let result = if push {
                navigator.push_with_query(&path, &query)
            } else {
                navigator.replace_with_query(&path, &query)
            };

            if let Err(error) = result {
                log::error!("Failed to set route: {error}");
            }
        }
    }

    fn handle_analysis(&mut self, ctx: &Context<Prompt>, analysis: Vec<String>) {
        if let Some(input) = analysis.get(0) {
            self.refresh(ctx, input);
        }

        if self.query.a != analysis || self.query.i != 0 {
            self.query.a = analysis;
            self.query.i = 0;
            self.save_query(ctx, true);
        }
    }
}

#[derive(Properties)]
pub(crate) struct Props {
    pub(crate) db: Arc<Option<lib::database::Database<'static>>>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.db, &other.db)
    }
}

impl Component for Prompt {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let handle = ctx
            .link()
            .add_location_listener(ctx.link().callback(Msg::HistoryChanged));
        let (query, inputs) = decode_query(ctx.link().location());

        let mut this = Self {
            query,
            entries: Default::default(),
            _handle: handle,
        };

        this.refresh(ctx, &inputs);
        this
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Error(error) => {
                log::error!("Failed to fetch: {error}");
                false
            }
            Msg::SearchResponse(response) => {
                self.entries = response
                    .entries
                    .into_iter()
                    .map(|e| (e.key, e.entry))
                    .collect();
                self.entries.sort_by(|(a, _), (b, _)| a.key.cmp(&b.key));
                true
            }
            Msg::AnalyzeResponse(response) => {
                let analysis = response.data.into_iter().map(|d| d.string).collect();
                self.handle_analysis(ctx, analysis);
                true
            }
            Msg::Mode(mode) => {
                self.query.mode = mode;

                self.query.q = match self.query.mode {
                    Mode::Unfiltered => self.query.q.clone(),
                    Mode::Hiragana => process_query(&self.query.q, romaji::Segment::hiragana),
                    Mode::Katakana => process_query(&self.query.q, romaji::Segment::katakana),
                };

                self.save_query(ctx, false);
                true
            }
            Msg::Change(input) => {
                let input = match self.query.mode {
                    Mode::Unfiltered => input,
                    Mode::Hiragana => process_query(&input, romaji::Segment::hiragana),
                    Mode::Katakana => process_query(&input, romaji::Segment::katakana),
                };

                self.refresh(ctx, &input);

                if self.query.q != input || !self.query.a.is_empty() {
                    self.query.q = input;
                    self.query.a.clear();
                    self.query.translation = None;
                    self.save_query(ctx, false);
                }

                true
            }
            Msg::ForceChange(input, translation) => {
                let input = match self.query.mode {
                    Mode::Unfiltered => input,
                    Mode::Hiragana => process_query(&input, romaji::Segment::hiragana),
                    Mode::Katakana => process_query(&input, romaji::Segment::katakana),
                };

                self.refresh(ctx, &input);

                self.query.q = input;
                self.query.translation = translation;
                self.query.a.clear();
                self.save_query(ctx, true);
                true
            }
            Msg::Analyze(i) => {
                if let Some(analysis) = self.analyze(ctx, i) {
                    if !analysis.is_empty() {
                        let analysis = analysis.into_values().collect::<Vec<_>>();
                        self.handle_analysis(ctx, analysis);
                    }
                }

                true
            }
            Msg::AnalyzeCycle => {
                if let Some(input) = self.query.a.get(self.query.i).cloned() {
                    self.query.i += 1;
                    self.query.i %= self.query.a.len();
                    self.save_query(ctx, true);
                    self.refresh(ctx, &input);
                    true
                } else {
                    false
                }
            }
            Msg::HistoryChanged(location) => {
                log::info!("history change");
                let (query, inputs) = decode_query(Some(location));
                self.query = query;
                self.refresh(ctx, &inputs);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().batch_callback(|e: InputEvent| {
            let input: HtmlInputElement = e.target_dyn_into()?;
            let value = input.value();
            Some(Msg::Change(value))
        });

        let onromanize = ctx
            .link()
            .batch_callback(|_: Event| Some(Msg::Mode(Mode::Unfiltered)));

        let onhiragana = ctx
            .link()
            .batch_callback(|_: Event| Some(Msg::Mode(Mode::Hiragana)));

        let onkatakana = ctx
            .link()
            .batch_callback(|_: Event| Some(Msg::Mode(Mode::Katakana)));

        let entries = (!self.entries.is_empty()).then(|| {
            let entries = c::entry::seq(self.entries.iter(), |(data, entry), not_last| {
                let entry: OwnedEntry = entry.clone();

                let change = ctx.link().callback(|(input, translation)| {
                    Msg::ForceChange(input, translation)
                });

                let entry = html!(<c::Entry sources={data.sources.clone()} entry_key={data.key.clone()} entry={entry} onchange={change} />);

                if not_last {
                    html!(<>{entry}<div class="entry-separator" /></>)
                } else {
                    entry
                }
            });

            html! {
                <div class="block block-lg">
                    <div class="entry-separator" />
                    {for entries}
                    <div class="entry-separator" />
                </div>
            }
        });

        let mut rem = 0;

        let analyze = if self.query.q.is_empty() {
            html! {
                <div class="block row analyze-text empty">{"Type something in the prompt"}</div>
            }
        } else {
            let query = self.query.q.char_indices().map(|(i, c)| {
                let sub = self.query.q.get(i..).unwrap_or_default();

                let event = if let Some(string) = self.query.a.get(self.query.i) {
                    if rem == 0 && sub.starts_with(string) {
                        rem = string.chars().count();
                        None
                    } else {
                        Some(i)
                    }
                } else {
                    Some(i)
                };

                let onclick = ctx.link().callback(move |e: MouseEvent| {
                    e.prevent_default();

                    match event {
                        Some(i) => Msg::Analyze(i),
                        None => Msg::AnalyzeCycle,
                    }
                });

                let class = classes! {
                    (rem > 0).then_some("active"),
                    (!(event.is_none() && self.query.a.len() <= 1)).then_some("clickable"),
                    "analyze-span"
                };

                rem = rem.saturating_sub(1);
                html!(<span {class} {onclick}>{c}</span>)
            });

            let analyze_hint = if self.query.a.len() > 1 {
                html! {
                    <div class="block row hint">
                        {format!("{} / {} (click character to cycle)", self.query.i + 1, self.query.a.len())}
                    </div>
                }
            } else if self.query.a.is_empty() {
                html! {
                    <div class="block row hint">
                        <span>{"Hint:"}</span>
                        {c::entry::spacing()}
                        <span>{"Click character for substring search"}</span>
                    </div>
                }
            } else {
                html!()
            };

            html! {
                <>
                    <div class="block row analyze-text">{for query}</div>
                    {analyze_hint}
                </>
            }
        };

        let analyze = html! {
            <div class="block block-lg indent" id="analyze">{analyze}</div>
        };

        let translation = self.query.translation.as_ref().map(|text| {
            html! {
                <div class="block row indent" id="translation">
                    <span class="translation-title">{"Translation:"}</span>
                    {c::entry::spacing()}
                    <span>{text}</span>
                </div>
            }
        });

        html! {
            <BrowserRouter>
                <div id="container">
                    <div class="block block row" id="prompt">
                        <input value={self.query.q.clone()} type="text" oninput={oninput} />
                    </div>

                    <div class="block block-lg row">
                        <label for="romanize" title="Do not process input at all">
                            <input type="checkbox" id="romanize" checked={self.query.mode == Mode::Unfiltered} onchange={onromanize} />
                            {"Default"}
                        </label>

                        {c::entry::spacing()}

                        <label for="hiragana" title="Process input as Hiragana">
                            <input type="checkbox" id="hiragana"  checked={self.query.mode == Mode::Hiragana} onchange={onhiragana} />
                            {"ひらがな"}
                        </label>

                        {c::entry::spacing()}

                        <label for="katakana" title="Treat input as Katakana">
                            <input type="checkbox" id="katakana" checked={self.query.mode == Mode::Katakana} onchange={onkatakana} />
                            {"カタカナ"}
                        </label>
                    </div>

                    <>
                        {analyze}
                        {for translation}
                        {for entries}
                    </>

                    <div class="block block-xl" id="copyright">{copyright()}</div>
                </div>
            </BrowserRouter>
        }
    }
}

fn process_query<'a, F>(input: &'a str, segment: F) -> String
where
    F: Copy + FnOnce(&romaji::Segment<'a>) -> &'a str,
{
    let mut out = String::new();

    for s in romaji::analyze(input) {
        out.push_str(segment(&s));
    }

    out
}

fn decode_query(location: Option<Location>) -> (Query, String) {
    let query = match location {
        Some(location) => location.query().ok(),
        None => None,
    };

    let query = query.unwrap_or_default();
    let query = Query::deserialize(query);

    let input = if query.a.is_empty() {
        query.q.clone()
    } else if let Some(input) = query.a.get(query.i) {
        input.clone()
    } else {
        query.q.clone()
    };

    (query, input)
}

fn copyright() -> Html {
    html! {
        <div class="block inline">
            <span>{"This application uses the JMdict dictionary file. "}</span>
            <span>{"This is the property of the "}</span>
            <a href="https://www.edrdg.org">{"Electronic Dictionary Research and Development Group"}</a>
            <span>{", and are used in conformance with the Group's "}</span>
            <a href="https://www.edrdg.org/edrdg/licence.html">{"licence"}</a>
            <span>{"."}</span>
        </div>
    }
}
