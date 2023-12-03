use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::str::from_utf8;

use lib::api;
use lib::api::ClientEvent;
use lib::kanjidic2;
use lib::romaji;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::{prelude::*, AnyRoute};

use crate::callbacks::Callbacks;
use crate::error::Error;
use crate::query::{Mode, Query, Tab};
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
    OpenConfig,
    Mode(Mode),
    CaptureClipboard(bool),
    Tab(Tab),
    Change(String),
    ForceChange(String, Option<String>),
    Analyze(usize),
    AnalyzeCycle,
    HistoryChanged(Location),
    SearchResponse(api::OwnedSearchResponse),
    AnalyzeResponse(api::OwnedAnalyzeResponse),
    MoreEntries,
    MoreCharacters,
    ClientEvent(ClientEvent),
    Error(Error),
}

impl From<Error> for Msg {
    #[inline]
    fn from(error: Error) -> Self {
        Msg::Error(error)
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
    log: Vec<api::LogEntry>,
    tasks: BTreeMap<String, api::TaskProgress>,
    analysis: Rc<[Rc<str>]>,
    _location_handle: Option<LocationHandle>,
}

impl Prompt {
    fn reload(&mut self, ctx: &Context<Self>) {
        log::trace!("Reload");

        if let Some(analyze_at) = self.query.analyze_at {
            self.analyze(ctx, analyze_at);
            return;
        }

        let input = if let Some(input) = self.analysis.get(self.query.index) {
            input.clone()
        } else {
            self.query.text.clone()
        };

        self.search(ctx, &input);
    }

    fn search(&mut self, ctx: &Context<Self>, input: &str) {
        log::trace!("Search `{input}`");

        let input = input.to_lowercase();
        let serial = self.serials.search();

        ctx.link().send_future(async move {
            match fetch::search(&input, serial).await {
                Ok(entries) => Msg::SearchResponse(entries),
                Err(error) => Msg::Error(error),
            }
        });
    }

    fn analyze(&mut self, ctx: &Context<Self>, start: usize) {
        log::trace!("Analyze {start}");

        let input = self.query.text.clone();
        let serial = self.serials.analyze();

        ctx.link().send_future(async move {
            match fetch::analyze(&input, start, serial).await {
                Ok(entries) => Msg::AnalyzeResponse(entries),
                Err(error) => Msg::Error(error),
            }
        });
    }

    fn save_query(&mut self, ctx: &Context<Prompt>, history: History) {
        let (Some(location), Some(navigator)) = (ctx.link().location(), ctx.link().navigator())
        else {
            return;
        };

        let path = location.path();
        let path = AnyRoute::new(path);

        let query = self.query.serialize(false);

        let result = match history {
            History::Push => navigator.push_with_query_and_state(&path, &query, IsInternal::new()),
            History::Replace => {
                navigator.replace_with_query_and_state(&path, &query, IsInternal::new())
            }
        };

        if let Err(error) = result {
            log::error!("Failed to set route: {error}");
        }
    }

    /// Update from what looks like JSON in a clipboard.
    fn update_from_clipboard_json(
        &mut self,
        ctx: &Context<Self>,
        json: &lib::api::SendClipboardJson,
    ) -> Result<(), Error> {
        if self.query.capture_clipboard && self.query.text.as_ref() != json.primary.as_str() {
            self.query.text = json.primary.clone().into();
            self.analysis = Rc::from([]);
            self.query.index = 0;
            self.query.translation = json.secondary.as_ref().filter(|s| !s.is_empty()).cloned();
            self.save_query(ctx, History::Push);
            self.search(ctx, &json.primary);
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

        if self.query.capture_clipboard && self.query.text.as_ref() != data {
            self.query.text = data.into();
            self.analysis = Rc::from([]);
            self.query.index = 0;
            self.query.translation = None;
            self.save_query(ctx, History::Push);
            self.search(ctx, data);
        }

        Ok(())
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct Props {
    pub(crate) callbacks: Callbacks,
}

impl Component for Prompt {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let location_handle = ctx
            .link()
            .add_location_listener(ctx.link().callback(Msg::HistoryChanged));

        let query = decode_query(ctx.link().location());

        let mut this = Self {
            query,
            phrases: Vec::default(),
            names: Vec::default(),
            limit_entries: DEFAULT_LIMIT,
            characters: Vec::default(),
            limit_characters: DEFAULT_LIMIT,
            serials: Serials::default(),
            log: Vec::new(),
            tasks: BTreeMap::new(),
            analysis: Rc::from([]),
            _location_handle: location_handle,
        };

        ctx.props()
            .callbacks
            .set_client_event(ctx.link().callback(Msg::ClientEvent));

        this.reload(ctx);
        this
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OpenConfig => {
                self.query.tab = Tab::Settings;
                self.save_query(ctx, History::Push);
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
            Msg::AnalyzeResponse(response) => {
                if response.serial == Some(self.serials.analyze) {
                    self.analysis = response.data.into_iter().map(|d| d.string.into()).collect();

                    if let Some(input) = self.analysis.get(self.query.index).cloned() {
                        self.search(ctx, &input);
                    }

                    true
                } else {
                    false
                }
            }
            Msg::Mode(mode) => {
                self.query.mode = mode;

                let new_query = match self.query.mode {
                    Mode::Unfiltered => self.query.text.clone(),
                    Mode::Hiragana => process_query(&self.query.text, romaji::Segment::hiragana),
                    Mode::Katakana => process_query(&self.query.text, romaji::Segment::katakana),
                };

                let history = if new_query != self.query.text {
                    History::Push
                } else {
                    History::Replace
                };

                self.query.text = new_query;
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
                    Mode::Unfiltered => Rc::from(input),
                    Mode::Hiragana => process_query(&input, romaji::Segment::hiragana),
                    Mode::Katakana => process_query(&input, romaji::Segment::katakana),
                };

                self.search(ctx, &input);

                if self.query.text != input || !self.analysis.is_empty() {
                    self.query.text = input;
                    self.analysis = Rc::from([]);
                    self.query.index = 0;
                    self.query.translation = None;
                    self.save_query(ctx, History::Replace);
                }

                true
            }
            Msg::ForceChange(input, translation) => {
                let input = match self.query.mode {
                    Mode::Unfiltered => Rc::from(input),
                    Mode::Hiragana => process_query(&input, romaji::Segment::hiragana),
                    Mode::Katakana => process_query(&input, romaji::Segment::katakana),
                };

                self.search(ctx, &input);

                self.query.text = input;
                self.query.translation = translation;
                self.analysis = Rc::from([]);
                self.query.index = 0;
                self.save_query(ctx, History::Push);
                true
            }
            Msg::Analyze(i) => {
                if self.query.analyze_at != Some(i) {
                    self.query.index = 0;
                }

                self.query.analyze_at = Some(i);
                self.save_query(ctx, History::Push);
                self.analyze(ctx, i);
                true
            }
            Msg::AnalyzeCycle => {
                if !self.analysis.is_empty() {
                    self.query.index += 1;
                    self.query.index %= self.analysis.len();
                }

                if let Some(input) = self.analysis.get(self.query.index).cloned() {
                    self.save_query(ctx, History::Push);
                    self.search(ctx, &input);
                    true
                } else {
                    false
                }
            }
            Msg::HistoryChanged(location) => {
                // Prevents internal history changes from firing.
                if location.state::<IsInternal>().filter(|s| s.set()).is_some() {
                    return false;
                }

                log::trace!("History change");
                let query = decode_query(Some(location));

                if self.query.analyze_at != query.analyze_at || self.query.text != query.text {
                    self.reload(ctx);
                } else if self.query.index != query.index {
                    if let Some(input) = self.analysis.get(self.query.index).cloned() {
                        self.search(ctx, &input);
                    }
                }

                self.query = query;
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
                    api::ClientEvent::LogBackFill(log) => {
                        self.log.extend(log.log);
                    }
                    api::ClientEvent::LogEntry(entry) => {
                        self.log.push(entry);
                    }
                    ClientEvent::TaskProgress(task) => {
                        self.tasks.insert(task.name.clone(), task);
                    }
                    ClientEvent::TaskCompleted(task) => {
                        self.tasks.remove(&task.name);
                    }
                    ClientEvent::Refresh(..) => {
                        self.reload(ctx);
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

    fn destroy(&mut self, ctx: &Context<Self>) {
        ctx.props().callbacks.clear_client_event();
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

        let analyze = if self.query.text.is_empty() {
            let text = if self.query.embed {
                "Nothing to analyze"
            } else {
                "Type something in the prompt"
            };

            html!(<div id="analyze" class="block row analyze-text empty">{text}</div>)
        } else {
            let on_analyze = ctx.link().callback(Msg::Analyze);
            let on_analyze_cycle = ctx.link().callback(|_| Msg::AnalyzeCycle);
            html!(<c::AnalyzeToggle query={self.query.text.clone()} analyzed={self.analysis.clone()} index={self.query.index} analyze_at={self.query.analyze_at} {on_analyze} {on_analyze_cycle} />)
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

            let header = (!self.query.embed).then(|| {
                html!(<h4>{"Kanji"}</h4>)
            });

            html! {
                <div class="block block-lg">
                    {header}
                    {for iter}
                    {for more}
                </div>
            }
        });

        let page = if self.query.embed {
            let tab = |title: &str, len: usize, tab: Tab| {
                let is_tab = self.query.tab == tab;
                let entries_classes = classes!(
                    "tab",
                    is_tab.then_some("active"),
                    (len == 0).then_some("disabled")
                );

                let onclick = (!is_tab).then(|| ctx.link().callback(move |_| Msg::Tab(tab)));

                let text = format!("{title} ({len})");

                html! {
                    <a class={entries_classes} {onclick}>{text}</a>
                }
            };

            let tabs = [
                tab("Phrases", self.phrases.len(), Tab::Phrases),
                tab("Names", self.names.len(), Tab::Names),
                tab("Kanji", self.characters.len(), Tab::Kanji),
            ];

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
                Tab::Settings => {
                    let onback = ctx.link().callback(|_| Msg::Tab(Tab::Phrases));
                    html!(<div class="block block-lg"><c::Config embed={self.query.embed} log={self.log.clone()} {onback} /></div>)
                }
            };

            html! {
                <>
                    <div class="block block-lg">{analyze}</div>
                    {for translation}
                    <div class="tabs">{for tabs}</div>
                    {content}
                </>
            }
        } else {
            match self.query.tab {
                Tab::Settings => {
                    let onback = ctx.link().callback(|_| Msg::Tab(Tab::Phrases));
                    html!(<div class="block block-lg"><c::Config embed={self.query.embed} log={self.log.clone()} {onback} /></div>)
                }
                _ => {
                    let onclick = ctx.link().callback(|_| Msg::OpenConfig);

                    let prompt = html! {
                        <>
                        <div class="block block row" id="prompt">
                            <input value={self.query.text.clone()} type="text" oninput={oninput} />
                        </div>

                        <div class="block block-lg row row-spaced">
                            <label for="romanize" title="Do not process input at all">
                                <input type="checkbox" id="romanize" checked={self.query.mode == Mode::Unfiltered} onchange={onromanize} />
                                {"Default"}
                            </label>

                            <label for="hiragana" title="Process input as Hiragana">
                                <input type="checkbox" id="hiragana" checked={self.query.mode == Mode::Hiragana} onchange={onhiragana} />
                                {"ひらがな"}
                            </label>

                            <label for="katakana" title="Treat input as Katakana">
                                <input type="checkbox" id="katakana" checked={self.query.mode == Mode::Katakana} onchange={onkatakana} />
                                {"カタカナ"}
                            </label>

                            <label for="clipboard" title="Capture clipboard">
                                <input type="checkbox" id="clipboard" checked={self.query.capture_clipboard} onchange={oncaptureclipboard} />
                                {"📋"}
                            </label>

                            <span class="end clickable" {onclick}>{"⚙ Config"}</span>
                        </div>
                        </>
                    };

                    let kanjis = kanjis.map(|kanjis| {
                        html! {
                            <div class="column characters">{kanjis}</div>
                        }
                    });

                    html! {
                        <>
                            <>{prompt}</>

                            <>
                                <div class="block block-xl">{analyze}</div>
                                {for translation}

                                <div class="columns">
                                    <div class="column">{phrases}{names}</div>
                                    {for kanjis}
                                </div>
                            </>
                        </>
                    }
                }
            }
        };

        let class = classes! {
            self.query.embed.then_some("embed"),
        };

        let tasks = (!self.tasks.is_empty()).then(|| {
            let tasks = self.tasks.values().map(|task| {
                let (progress, done, value) = match task.total {
                    Some(total) => {
                        let progress = html! {
                            <progress max={total.to_string()} value={task.value.to_string()} />
                        };

                        (progress, task.value == total, None)
                    }
                    None => {
                        let progress = html!(<progress />);
                        let value = html!(<div class="task-field task-value">{task.value.to_string()}</div>);
                        (progress, false, Some(value))
                    }
                };

                let class = classes! {
                    "block",
                    "row",
                    "row-spaced",
                    "task",
                    done.then_some("done"),
                };

                let progress_text = format!("{} ...", task.text);

                let text = (!self.query.embed).then(|| {
                    html!(<div class="task-field task-text">{progress_text.clone()}</div>)
                });

                html! {
                    <div {class} title={progress_text}>
                        <div class="task-field task-name">{&task.name}</div>
                        <div class="task-field task-step">{format!("{}/{}", task.step, task.steps)}</div>
                        {text}
                        <div class="task-field task-progress">{progress}</div>
                        {value}
                    </div>
                }
            });

            html! {
                <div class="block block-lg" id="tasks">
                    {for tasks}
                </div>
            }
        });

        let window_top = self.query.embed.then(|| {
            let onclick = ctx.link().callback(|_| Msg::Tab(Tab::Settings));

            let config = html! {
                <a class="config clickable" {onclick}>{"⚙"}</a>
            };

            let maximize = self.query.to_href(true).map(|href| {
                html! {
                    <a class="maximize clickable" {href} target="_window">{"🗖"}</a>
                }
            });

            html! {
                <div id="window-top">
                    <div class="window-title">{"jpv"}</div>
                    {config}
                    {maximize}
                </div>
            }
        });

        html! {
            <>
                {window_top}

                <div id="container" {class}>
                    {tasks}
                    {page}
                    <div class="block block-xl" id="copyright">{copyright()}</div>
                </div>
            </>
        }
    }
}

fn process_query<'a, F>(input: &'a str, segment: F) -> Rc<str>
where
    F: Copy + FnOnce(&romaji::Segment<'a>) -> &'a str,
{
    let mut out = String::new();

    for s in romaji::analyze(input) {
        out.push_str(segment(&s));
    }

    Rc::from(out)
}

fn decode_query(location: Option<Location>) -> Query {
    let query = match location {
        Some(location) => location.query().ok(),
        None => None,
    };

    let query = query.unwrap_or_default();
    let (mut query, analyze_at_char) = Query::deserialize(query);

    if let Some(analyze_at_char) = analyze_at_char {
        let mut len = 0;

        for c in query.text.chars().take(analyze_at_char) {
            len += c.len_utf8();
        }

        query.analyze_at = Some(len);
    };

    query
}

fn copyright() -> Html {
    html! {
        <>
            <div class="block inline">
                <span>{"Made with ❤️ by "}</span>
                <a href="https://udoprog.github.io">{"John-John Tedro"}</a>
                <span>{", freely available forever under the "}</span>
                <a href="https://github.com/udoprog/jpv/blob/main/LICENSE-MIT">{"MIT"}</a>
                <span>{" or "}</span>
                <a href="https://github.com/udoprog/jpv/blob/main/LICENSE-APACHE">{"Apache 2.0 license"}</a>
            </div>

            <div class="block inline">
                <span>{"This application uses "}</span>
                <a href="https://www.edrdg.org/wiki/index.php/JMdict-EDICT_Dictionary_Project">{"JMDICT"}</a>
                <span>{", "}</span>
                <a href="https://www.edrdg.org/wiki/index.php/KANJIDIC_Project">{"KANJIDIC2"}</a>
                <span>{", and "}</span>
                <a href="http://edrdg.org/enamdict/enamdict_doc.html">{"ENAMDICT"}</a>
                <span>{" which is the property of the "}</span>
                <a href="https://www.edrdg.org">{"EDRDG"}</a>
                <span>{" and is used in conformance with its "}</span>
                <a href="https://www.edrdg.org/edrdg/licence.html">{"licence"}</a>
                <span>{"."}</span>
            </div>
        </>
    }
}

/// Internal state for the history API, so it can be read by the listener and
/// avoid double-querying.
struct IsInternal(Cell<bool>);

impl IsInternal {
    #[inline]
    fn new() -> Self {
        Self(Cell::new(true))
    }

    #[inline]
    fn set(&self) -> bool {
        let old = self.0.get();
        self.0.set(false);
        old
    }
}
