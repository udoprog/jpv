use std::borrow::Cow;
use std::str::from_utf8;
use std::sync::Arc;

use lib::api;
use lib::database::EntryResultKey;
use lib::jmdict;
use lib::kanjidic2;
use lib::romaji;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::{prelude::*, AnyRoute};

const DEFAULT_LIMIT: usize = 100;

use crate::c::entry::seq;
use crate::error::Error;
use crate::ws;
use crate::{components as c, fetch};

pub(crate) enum Msg {
    Mode(Mode),
    CaptureClipboard(bool),
    Tab(Tab),
    Change(String),
    ForceChange(String, Option<String>),
    Analyze(usize),
    AnalyzeCycle,
    HistoryChanged(Location),
    SearchResponse(fetch::SearchResponse),
    AnalyzeResponse(fetch::AnalyzeResponse),
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
    Entries,
    Characters,
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
                "cb" => {
                    this.capture_clipboard = value == "yes";
                }
                "embed" => {
                    this.embed = value == "yes";
                }
                "tab" => {
                    this.tab = match value.as_str() {
                        "characters" => Tab::Characters,
                        "entries" => Tab::Entries,
                        _ => Tab::default(),
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

        if self.capture_clipboard {
            out.push(("cb", Cow::Borrowed("yes")));
        }

        if self.embed {
            out.push(("embed", Cow::Borrowed("yes")));
        }

        match self.tab {
            Tab::Entries => {}
            Tab::Characters => {
                out.push(("tab", Cow::Borrowed("characters")));
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
    entries: Vec<(EntryResultKey, jmdict::OwnedEntry)>,
    limit_entries: usize,
    characters: Vec<kanjidic2::OwnedCharacter>,
    limit_characters: usize,
    serials: Serials,
    ws: ws::Service<Self>,
    _handle: Option<LocationHandle>,
}

impl Prompt {
    fn refresh(&mut self, ctx: &Context<Self>, input: &str) {
        if let Some(db) = &*ctx.props().db {
            let input = input.to_lowercase();

            ctx.link().send_message(match db.search(&input) {
                Ok(search) => {
                    let entries = search
                        .entries
                        .into_iter()
                        .map(|(key, e)| fetch::SearchEntry {
                            key,
                            entry: borrowme::to_owned(e),
                        })
                        .collect();

                    let characters = search
                        .characters
                        .into_iter()
                        .map(borrowme::to_owned)
                        .collect();

                    let serial = self.serials.search();

                    let response = fetch::SearchResponse {
                        entries,
                        characters,
                        serial,
                    };

                    Msg::SearchResponse(response)
                }
                Err(error) => Msg::Error(error.into()),
            });
        } else {
            let input = input.to_lowercase();
            let serial = self.serials.search();

            ctx.link().send_future(async move {
                match fetch::search(&input, serial).await {
                    Ok(entries) => Msg::SearchResponse(entries),
                    Err(error) => Msg::Error(error),
                }
            });
        }
    }

    fn analyze(&mut self, ctx: &Context<Self>, start: usize) {
        if let Some(db) = &*ctx.props().db {
            let serial = self.serials.analyze();

            ctx.link()
                .send_message(match db.analyze(&self.query.q, start) {
                    Ok(data) => Msg::AnalyzeResponse(fetch::AnalyzeResponse {
                        data: data
                            .into_iter()
                            .map(|(key, string)| fetch::AnalyzeEntry { key, string })
                            .collect(),
                        serial,
                    }),
                    Err(error) => Msg::Error(error.into()),
                });
        } else {
            let input = self.query.q.clone();
            let serial = self.serials.analyze();

            ctx.link().send_future(async move {
                match fetch::analyze(&input, start, serial).await {
                    Ok(entries) => Msg::AnalyzeResponse(entries),
                    Err(error) => Msg::Error(error),
                }
            });
        }
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
            self.save_query(ctx, true);
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
            self.save_query(ctx, true);
            self.refresh(ctx, data);
        }

        Ok(())
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
            entries: Vec::default(),
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

        this.refresh(ctx, &inputs);
        this
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SearchResponse(response) => {
                if response.serial == self.serials.search {
                    self.entries = response
                        .entries
                        .into_iter()
                        .map(|e| (e.key, e.entry))
                        .collect();
                    self.entries.sort_by(|(a, _), (b, _)| a.key.cmp(&b.key));
                    self.characters = response.characters;
                    self.limit_entries = DEFAULT_LIMIT;
                    self.limit_characters = DEFAULT_LIMIT;
                    true
                } else {
                    false
                }
            }
            Msg::AnalyzeResponse(response) => {
                if response.serial == self.serials.analyze {
                    let analysis = response.data.into_iter().map(|d| d.string).collect();
                    self.handle_analysis(ctx, analysis);
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

                let is_changed = new_query != self.query.q;
                self.query.q = new_query;
                self.save_query(ctx, is_changed);
                true
            }
            Msg::CaptureClipboard(capture_clipboard) => {
                self.query.capture_clipboard = capture_clipboard;
                self.save_query(ctx, false);
                true
            }
            Msg::Tab(tab) => {
                self.query.tab = tab;
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
                self.analyze(ctx, i);
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
            <div class="block block-xl" id="analyze">{analyze}</div>
        };

        let translation = self.query.translation.as_ref().map(|text| {
            html! {
                <div class="block row" id="translation">
                    <span class="translation-title">{"Translation:"}</span>
                    {c::entry::spacing()}
                    <span>{text}</span>
                </div>
            }
        });

        let entries = (!self.entries.is_empty()).then(|| {
            let entries = seq(self.entries.iter().take(self.limit_entries), |(data, entry), not_last| {
                let entry: jmdict::OwnedEntry = entry.clone();

                let change = ctx.link().callback(|(input, translation)| {
                    Msg::ForceChange(input, translation)
                });

                let entry = html!(<c::Entry embed={self.query.embed} sources={data.sources.clone()} entry_key={data.key} entry={entry} onchange={change} />);

                if not_last {
                    html!(<>{entry}<div class="entry-separator" /></>)
                } else {
                    entry
                }
            });

            let more = (self.entries.len() > self.limit_entries).then(|| {
                html! {
                    <div class="block block-lg">
                        <div class="block row">
                            {format!("Showing {} out of {} entries", self.limit_entries, self.entries.len())}
                        </div>

                        <div class="block row">
                            <button class="btn" onclick={ctx.link().callback(|_| Msg::MoreEntries)}>{"Show more"}</button>
                        </div>
                    </div>
                }
            });

            let header = if self.query.embed {
                None
            } else {
                Some(html!(<h4>{"Entries"}</h4>))
            };

            html! {
                <div class="block block-lg">
                    {for header}
                    {for entries}
                    {for more}
                </div>
            }
        });

        let characters = (!self.characters.is_empty()).then(|| {
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
                Some(html!(<h4>{"Characters"}</h4>))
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

            tabs.push(tab("Entries", self.entries.len(), Tab::Entries));
            tabs.push(tab("Characters", self.characters.len(), Tab::Characters));

            let content = match self.query.tab {
                Tab::Entries => {
                    html!(<div class="block block-lg">{entries}</div>)
                }
                Tab::Characters => {
                    html!(<div class="block block-lg characters">{characters}</div>)
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
                    <div class="column">{entries}</div>
                    <div class="column characters">{characters}</div>
                </div>
            }
        };

        let class = classes! {
            self.query.embed.then_some("embed"),
        };

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

                {c::entry::spacing()}

                <label for="clipboard" title="Capture clipboard">
                    <input type="checkbox" id="clipboard" checked={self.query.capture_clipboard} onchange={oncaptureclipboard} />
                    {"📋"}
                </label>
            </div>
            </>
        });

        html! {
            <BrowserRouter>
                <div id="container" {class}>
                    <>{for prompt}</>

                    <>
                        {analyze}
                        {for translation}
                        {results}
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
