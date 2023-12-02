use std::borrow::Cow;
use std::str::from_utf8;

use lib::api;
use lib::kanjidic2;
use lib::romaji;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::{prelude::*, AnyRoute};

use crate::error::Error;
use crate::ws;
use crate::Route;
use crate::{components as c, fetch};

use super::{comma, seq, spacing};

const DEFAULT_LIMIT: usize = 100;

// How a history update is performed
pub(crate) enum History {
    /// History is pushed.
    Push,
    /// History is replaced.
    Replace,
}

pub(crate) enum Msg {
    Navigate(Route),
    Mode(Mode),
    CaptureClipboard(bool),
    Tab(Tab),
    Change(String),
    ForceChange(String, Option<String>),
    Analyze(usize),
    AnalyzeCycle,
    HistoryChanged(Location),
    SearchResponse(api::OwnedSearchResponse),
    AnalyzeResponse(api::OwnedAnalyzeResponse, History),
    MoreEntries,
    MoreCharacters,
    WebSocket(ws::Msg),
    ClientEvent(api::ClientEvent),
    Error(Error),
}

impl From<ws::Msg> for Msg {
    #[inline]
    fn from(msg: ws::Msg) -> Self {
        Msg::WebSocket(msg)
    }
}

impl From<api::ClientEvent> for Msg {
    #[inline]
    fn from(msg: api::ClientEvent) -> Self {
        Msg::ClientEvent(msg)
    }
}

impl From<Error> for Msg {
    #[inline]
    fn from(error: Error) -> Self {
        Msg::Error(error)
    }
}

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
}

#[derive(Default, Debug)]
struct Query {
    q: String,
    translation: Option<String>,
    a: Vec<String>,
    i: usize,
    mode: Mode,
    capture_clipboard: bool,
    embed: bool,
    tab: Tab,
}

impl Query {
    fn deserialize(raw: Vec<(String, String)>) -> (Self, Option<usize>) {
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
        }

        out
    }
}

#[derive(Default)]
struct Serials {
    search: u32,
    analyze: u32,
}

impl Serials {
    fn search(&mut self) -> u32 {
        self.search = self.search.wrapping_add(1);
        self.search
    }

    fn analyze(&mut self) -> u32 {
        self.analyze = self.analyze.wrapping_add(1);
        self.analyze
    }
}

pub(crate) struct Prompt {
    query: Query,
    phrases: Vec<api::OwnedSearchPhrase>,
    names: Vec<api::OwnedSearchName>,
    limit_entries: usize,
    characters: Vec<kanjidic2::OwnedCharacter>,
    limit_characters: usize,
    serials: Serials,
    ws: ws::Service<Self>,
    _handle: Option<LocationHandle>,
}

impl Prompt {
    fn refresh(&mut self, ctx: &Context<Self>, input: &str) {
        let input = input.to_lowercase();
        let serial = self.serials.search();

        ctx.link().send_future(async move {
            match fetch::search(&input, serial).await {
                Ok(entries) => Msg::SearchResponse(entries),
                Err(error) => Msg::Error(error),
            }
        });
    }

    fn analyze(&mut self, ctx: &Context<Self>, start: usize, history: History) {
        let input = self.query.q.clone();
        let serial = self.serials.analyze();

        ctx.link().send_future(async move {
            match fetch::analyze(&input, start, serial).await {
                Ok(entries) => Msg::AnalyzeResponse(entries, history),
                Err(error) => Msg::Error(error),
            }
        });
    }

    fn save_query(&mut self, ctx: &Context<Prompt>, history: History) {
        if let (Some(location), Some(navigator)) = (ctx.link().location(), ctx.link().navigator()) {
            let path = location.path();
            let path = AnyRoute::new(path);

            let query = self.query.serialize();

            let result = match history {
                History::Push => navigator.push_with_query(&path, &query),
                History::Replace => navigator.replace_with_query(&path, &query),
            };

            if let Err(error) = result {
                log::error!("Failed to set route: {error}");
            }
        }
    }

    fn handle_analysis(&mut self, ctx: &Context<Prompt>, analysis: Vec<String>, history: History) {
        if let Some(input) = analysis.get(0) {
            self.refresh(ctx, input);
        }

        if self.query.a != analysis || self.query.i != 0 {
            self.query.a = analysis;
            self.query.i = 0;
            self.save_query(ctx, history);
        }
    }

    /// Update from what looks like JSON in a clipboard.
    fn update_from_clipboard_json(
        &mut self,
        ctx: &Context<Self>,
        json: &lib::api::SendClipboardJson,
    ) -> Result<(), Error> {
        if self.query.capture_clipboard && self.query.q != json.primary {
            self.query.q = json.primary.clone();
            self.query.a.clear();
            self.query.translation = json.secondary.as_ref().filter(|s| !s.is_empty()).cloned();
            self.save_query(ctx, History::Push);
            self.refresh(ctx, &json.primary);
        }

        Ok(())
    }

    /// Update from clipboard.
    fn update_from_clipboard(
        &mut self,
        ctx: &Context<Self>,
        ty: Option<&str>,
        data: &[u8],
    ) -> Result<(), Error> {
        if matches!(ty, Some("application/json")) {
            let json = serde_json::from_slice::<lib::api::SendClipboardJson>(data)?;
            self.update_from_clipboard_json(ctx, &json)?;
            return Ok(());
        }

        // Heuristics.
        if data.starts_with(&[b'{']) {
            if let Ok(json) = serde_json::from_slice::<lib::api::SendClipboardJson>(data) {
                self.update_from_clipboard_json(ctx, &json)?;
                return Ok(());
            }
        }

        let data = from_utf8(data)?;

        if self.query.capture_clipboard && self.query.q != data {
            self.query.q = data.to_owned();
            self.query.a.clear();
            self.query.translation = None;
            self.save_query(ctx, History::Push);
            self.refresh(ctx, data);
        }

        Ok(())
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct Props;

impl Component for Prompt {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let handle = ctx
            .link()
            .add_location_listener(ctx.link().callback(Msg::HistoryChanged));

        let (query, input, analyze_at) = decode_query(ctx.link().location());

        let mut this = Self {
            query,
            phrases: Vec::default(),
            names: Vec::default(),
            limit_entries: DEFAULT_LIMIT,
            characters: Vec::default(),
            limit_characters: DEFAULT_LIMIT,
            serials: Serials::default(),
            ws: ws::Service::new(),
            _handle: handle,
        };

        if !this.query.embed {
            if let Err(error) = this.ws.connect(ctx) {
                ctx.link().send_message(error);
            }
        }

        if let Some(analyze_at) = analyze_at {
            this.analyze(ctx, analyze_at, History::Replace);
        } else {
            this.refresh(ctx, &input);
        }

        this
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Navigate(route) => {
                if let Some(navigator) = ctx.link().navigator() {
                    navigator.push(&route);
                }

                true
            }
            Msg::SearchResponse(response) => {
                if response.serial == Some(self.serials.search) {
                    self.phrases = response.phrases;
                    self.names = response.names;
                    self.phrases.sort_by(|a, b| a.key.weight.cmp(&b.key.weight));
                    self.names.sort_by(|a, b| a.key.weight.cmp(&b.key.weight));
                    self.characters = response.characters;
                    self.limit_entries = DEFAULT_LIMIT;
                    self.limit_characters = DEFAULT_LIMIT;
                    true
                } else {
                    false
                }
            }
            Msg::AnalyzeResponse(response, history) => {
                if response.serial == Some(self.serials.analyze) {
                    let analysis = response.data.into_iter().map(|d| d.string).collect();
                    self.handle_analysis(ctx, analysis, history);
                    true
                } else {
                    false
                }
            }
            Msg::Mode(mode) => {
                self.query.mode = mode;

                let new_query = match self.query.mode {
                    Mode::Unfiltered => self.query.q.clone(),
                    Mode::Hiragana => process_query(&self.query.q, romaji::Segment::hiragana),
                    Mode::Katakana => process_query(&self.query.q, romaji::Segment::katakana),
                };

                let history = if new_query != self.query.q {
                    History::Push
                } else {
                    History::Replace
                };

                self.query.q = new_query;
                self.save_query(ctx, history);
                true
            }
            Msg::CaptureClipboard(capture_clipboard) => {
                self.query.capture_clipboard = capture_clipboard;
                self.save_query(ctx, History::Replace);
                true
            }
            Msg::Tab(tab) => {
                self.query.tab = tab;
                self.save_query(ctx, History::Replace);
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
                    self.save_query(ctx, History::Replace);
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
                self.save_query(ctx, History::Push);
                true
            }
            Msg::Analyze(i) => {
                self.analyze(ctx, i, History::Push);
                true
            }
            Msg::AnalyzeCycle => {
                if let Some(input) = self.query.a.get(self.query.i).cloned() {
                    self.query.i += 1;
                    self.query.i %= self.query.a.len();
                    self.save_query(ctx, History::Push);
                    self.refresh(ctx, &input);
                    true
                } else {
                    false
                }
            }
            Msg::HistoryChanged(location) => {
                log::info!("history change");

                let (query, inputs, analyze_at) = decode_query(Some(location));
                self.query = query;

                if let Some(analyze_at) = analyze_at {
                    self.analyze(ctx, analyze_at, History::Replace);
                } else {
                    self.refresh(ctx, &inputs);
                }

                true
            }
            Msg::MoreEntries => {
                self.limit_entries += DEFAULT_LIMIT;
                true
            }
            Msg::MoreCharacters => {
                self.limit_characters += DEFAULT_LIMIT;
                true
            }
            Msg::WebSocket(msg) => {
                self.ws.update(ctx, msg);
                false
            }
            Msg::ClientEvent(event) => {
                match event {
                    api::ClientEvent::SendClipboardData(clipboard) => {
                        if let Err(error) = self.update_from_clipboard(
                            ctx,
                            clipboard.ty.as_deref(),
                            &clipboard.data,
                        ) {
                            ctx.link().send_message(error);
                        }
                    }
                }

                true
            }
            Msg::Error(error) => {
                log::error!("Failed to fetch: {error}");
                false
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

        let oncaptureclipboard = ctx.link().batch_callback({
            let capture_clipboard = self.query.capture_clipboard;
            move |_: Event| Some(Msg::CaptureClipboard(!capture_clipboard))
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
                        {spacing()}
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
            <div class="block block-xl" id="analyze">{analyze}</div>
        };

        let translation = self.query.translation.as_ref().map(|text| {
            html! {
                <div class="block row" id="translation">
                    <span class="translation-title">{"Translation:"}</span>
                    {spacing()}
                    <span>{text}</span>
                </div>
            }
        });

        let phrases = (!self.phrases.is_empty()).then(|| {
            let phrases = self.phrases.iter().take(self.limit_entries).map(|e| {
                let entry = e.phrase.clone();

                let change = ctx.link().callback(|(input, translation)| {
                    Msg::ForceChange(input, translation)
                });

                html!(<c::Entry embed={self.query.embed} sources={e.key.sources.clone()} entry={entry} onchange={change} />)
            });

            let phrases = seq(phrases, |entry, not_last| {
                if not_last {
                    html!(<>{entry}<div class="entry-separator" /></>)
                } else {
                    entry
                }
            });

            let more = (self.phrases.len() > self.limit_entries).then(|| {
                html! {
                    <div class="block block-lg">
                        <div class="block row">
                            {format!("Showing {} out of {} phrases", self.limit_entries, self.phrases.len())}
                        </div>

                        <div class="block row">
                            <button class="btn" onclick={ctx.link().callback(|_| Msg::MoreEntries)}>{"Show more"}</button>
                        </div>
                    </div>
                }
            });

            let header = (!self.query.embed).then(|| {
                html!(<h4>{"Phrases"}</h4>)
            });

            html! {
                <div class="block block-lg">
                    {header}
                    {for phrases}
                    {for more}
                </div>
            }
        });

        let names = (!self.names.is_empty()).then(|| {
            let names = seq(self.names.iter(), |e, not_last| {
                let entry = html!(<c::Name embed={self.query.embed} sources={e.key.sources.clone()} entry={e.name.clone()} />);

                if not_last {
                    html!(<>{entry}{comma()}</>)
                } else {
                    entry
                }
            });

            let header = (!self.query.embed).then(|| {
                html!(<h4>{"Names"}</h4>)
            });

            html! {
                <>
                {header}
                <div class="block-lg row">{for names}</div>
                </>
            }
        });

        let kanjis = (!self.characters.is_empty()).then(|| {
            let iter = seq(self.characters.iter().take(self.limit_characters), |c, not_last| {
                let separator = not_last.then(|| html!(<div class="character-separator" />));

                html! {
                    <>
                        <c::Character embed={self.query.embed} character={c.clone()} />
                        {for separator}
                    </>
                }
            });

            let more = (self.characters.len() > self.limit_characters).then(|| {
                html! {
                    <div class="block block-lg">
                        <div class="block row">
                            {format!("Showing {} out of {} characters", self.limit_characters, self.characters.len())}
                        </div>

                        <div class="block row">
                            <button class="btn" onclick={ctx.link().callback(|_| Msg::MoreCharacters)}>{"Show more"}</button>
                        </div>
                    </div>
                }
            });

            let header = if self.query.embed {
                None
            } else {
                Some(html!(<h4>{"Kanji"}</h4>))
            };

            html! {
                <div class="block block-lg">
                    {for header}
                    {for iter}
                    {for more}
                </div>
            }
        });

        let results = if self.query.embed {
            let tab = |title: &str, len: usize, tab: Tab| {
                let is_tab = self.query.tab == tab;
                let entries_classes = classes!(
                    "tab",
                    is_tab.then_some("active"),
                    (len == 0).then_some("disabled")
                );

                let onclick = (!is_tab && len > 0).then(|| {
                    ctx.link().callback(move |e: MouseEvent| {
                        e.prevent_default();
                        Msg::Tab(tab)
                    })
                });

                html! {
                    <a class={entries_classes} {onclick}>{format!("{title} ({len})")}</a>
                }
            };

            let mut tabs = Vec::new();

            tabs.push(tab("Phrases", self.phrases.len(), Tab::Phrases));
            tabs.push(tab("Names", self.names.len(), Tab::Names));
            tabs.push(tab("Kanji", self.characters.len(), Tab::Kanji));

            let content = match self.query.tab {
                Tab::Phrases => {
                    html!(<div class="block block-lg">{phrases}</div>)
                }
                Tab::Names => {
                    html!(<div class="block block-lg">{names}</div>)
                }
                Tab::Kanji => {
                    html!(<div class="block block-lg kanjis">{kanjis}</div>)
                }
            };

            if tabs.is_empty() {
                content
            } else {
                html! {
                    <>
                        <div class="tabs">{for tabs}</div>
                        {content}
                    </>
                }
            }
        } else {
            html! {
                <div class="columns">
                    <div class="column">{phrases}{names}</div>
                    <div class="column characters">{kanjis}</div>
                </div>
            }
        };

        let class = classes! {
            self.query.embed.then_some("embed"),
        };

        let onclick = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::Navigate(Route::Config)
        });

        let prompt = (!self.query.embed).then(|| html! {
            <>
            <div class="block block row" id="prompt">
                <input value={self.query.q.clone()} type="text" oninput={oninput} />
            </div>

            <div class="block block-lg row">
                <label for="romanize" title="Do not process input at all">
                    <input type="checkbox" id="romanize" checked={self.query.mode == Mode::Unfiltered} onchange={onromanize} />
                    {"Default"}
                </label>

                {spacing()}

                <label for="hiragana" title="Process input as Hiragana">
                    <input type="checkbox" id="hiragana" checked={self.query.mode == Mode::Hiragana} onchange={onhiragana} />
                    {"ひらがな"}
                </label>

                {spacing()}

                <label for="katakana" title="Treat input as Katakana">
                    <input type="checkbox" id="katakana" checked={self.query.mode == Mode::Katakana} onchange={onkatakana} />
                    {"カタカナ"}
                </label>

                {spacing()}

                <label for="clipboard" title="Capture clipboard">
                    <input type="checkbox" id="clipboard" checked={self.query.capture_clipboard} onchange={oncaptureclipboard} />
                    {"📋"}
                </label>

                <button class="btn btn-lg end" {onclick}>{"⚙️"}</button>
            </div>
            </>
        });

        html! {
            <div id="container" {class}>
                <>{for prompt}</>

                <>
                    {analyze}
                    {for translation}
                    {results}
                </>

                <div class="block block-xl" id="copyright">{copyright()}</div>
            </div>
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

fn decode_query(location: Option<Location>) -> (Query, String, Option<usize>) {
    let query = match location {
        Some(location) => location.query().ok(),
        None => None,
    };

    let query = query.unwrap_or_default();
    let (query, analyze_at) = Query::deserialize(query);

    let input = if query.a.is_empty() {
        query.q.clone()
    } else if let Some(input) = query.a.get(query.i) {
        input.clone()
    } else {
        query.q.clone()
    };

    let analyze_at = match analyze_at {
        Some(analyze_at) => {
            let mut len = 0;

            for c in input.chars().take(analyze_at) {
                len += c.len_utf8();
            }

            Some(len)
        }
        None => None,
    };

    (query, input, analyze_at)
}

fn copyright() -> Html {
    html! {
        <>
            <div class="block inline">
                <span>{"Made with ❤️ by "}</span>
                <a href="https://udoprog.github.io">{"John-John Tedro"}</a>
                <span>{" made freely available under the "}</span>
                <a href="https://github.com/udoprog/jpv/blob/main/LICENSE-MIT">{"MIT"}</a>
                <span>{" or "}</span>
                <a href="https://github.com/udoprog/jpv/blob/main/LICENSE-APACHE">{"Apache 2.0 license"}</a>
            </div>

            <div class="block inline">
                <span>{"This application uses the JMdict dictionary file. "}</span>
                <span>{"This is the property of the "}</span>
                <a href="https://www.edrdg.org">{"Electronic Dictionary Research and Development Group"}</a>
                <span>{", and are used in conformance with the Group's "}</span>
                <a href="https://www.edrdg.org/edrdg/licence.html">{"licence"}</a>
                <span>{"."}</span>
            </div>
        </>
    }
}
