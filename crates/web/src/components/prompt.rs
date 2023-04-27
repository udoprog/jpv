use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use lib::database::IndexExtra;
use lib::elements::{Entry, EntryKey};
use lib::romaji;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::{prelude::*, AnyRoute};

use crate::components as c;

pub(crate) enum Msg {
    Change(String),
    Analyze(usize),
    AnalyzeCycle,
    HistoryChanged(Location),
}

#[derive(Default, Debug)]
struct Query {
    q: String,
    a: Vec<String>,
    i: usize,
}

impl Query {
    fn deserialize(raw: Vec<(String, String)>) -> Self {
        let mut this = Self::default();

        for (key, value) in raw {
            match key.as_str() {
                "q" => {
                    this.q = value;
                }
                "a" => {
                    this.a.push(value);
                }
                "i" => {
                    if let Ok(i) = value.parse() {
                        this.i = i;
                    }
                }
                _ => {}
            }
        }

        this
    }

    fn serialize(&self) -> Vec<(&'static str, Cow<'_, str>)> {
        let mut out = Vec::new();
        out.push(("q", Cow::Borrowed(self.q.as_str())));

        for a in &self.a {
            out.push(("a", Cow::Borrowed(a.as_str())));
        }

        if self.i != 0 {
            out.push(("i", Cow::Owned(self.i.to_string())));
        }

        out
    }
}

struct EntryData {
    extras: BTreeSet<IndexExtra>,
    len: usize,
    key: EntryKey,
}

#[derive(Default)]
pub(crate) struct Prompt {
    query: Query,
    entries: Vec<(EntryData, Entry<'static>)>,
    _handle: Option<LocationHandle>,
}

impl Prompt {
    fn refresh(&mut self, ctx: &Context<Self>, input: &str) {
        self.entries.clear();
        let mut dedup = HashMap::new();

        for id in ctx.props().db.lookup(input) {
            let Ok(entry) = ctx.props().db.get(id) else {
                continue;
            };

            let Some(&i) = dedup.get(&id.index()) else {
                dedup.insert(id.index(), self.entries.len());

                let data = EntryData {
                    extras: [id.extra()].into_iter().collect(),
                    len: input.chars().count(),
                    key: EntryKey::default(),
                };

                self.entries.push((data, entry));
                continue;
            };

            let Some((data, _)) = self.entries.get_mut(i) else {
                continue;
            };

            data.len = data.len.max(input.chars().count());
            data.extras.insert(id.extra());
        }

        for (data, e) in &mut self.entries {
            let conjugation = data.extras.iter().any(|index| match index {
                IndexExtra::VerbInflection(..) => true,
                IndexExtra::AdjectiveInflection(..) => true,
                _ => false,
            });

            data.key = e.sort_key(input, conjugation, data.len);
        }

        self.entries.sort_by(|a, b| a.0.key.cmp(&b.0.key));
    }

    fn analyze(&mut self, ctx: &Context<Self>, start: usize) -> BTreeSet<String> {
        let mut inputs = BTreeSet::new();

        let Some(suffix) = self.query.q.get(start..) else {
            return inputs;
        };

        let mut it = suffix.chars();

        while !it.as_str().is_empty() {
            if ctx.props().db.contains(it.as_str()) {
                inputs.insert(it.as_str().to_owned());
            }

            it.next_back();
        }

        inputs
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
}

#[derive(Properties)]
pub(crate) struct Props {
    pub(crate) db: Arc<lib::database::Database<'static>>,
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
            Msg::Change(input) => {
                let input = process_query(&input);
                self.refresh(ctx, &input);

                if self.query.q != input || !self.query.a.is_empty() {
                    self.query.q = input;
                    self.query.a.clear();
                    self.save_query(ctx, false);
                }

                true
            }
            Msg::Analyze(i) => {
                match self.analyze(ctx, i) {
                    analyze if !analyze.is_empty() => {
                        let analyze = analyze.into_iter().rev().collect::<Vec<_>>();

                        if let Some(input) = analyze.get(0) {
                            self.refresh(ctx, input);
                        }

                        if self.query.a != analyze || self.query.i != 0 {
                            self.query.a = analyze;
                            self.query.i = 0;
                            self.save_query(ctx, true);
                        }
                    }
                    _ => {}
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

        let entries = (!self.entries.is_empty()).then(|| {
            let entries = self.entries.iter().map(|(data, entry)| {
                html!(<c::Entry extras={data.extras.clone()} entry_key={data.key.clone()} entry={entry.clone()} />)
            });

            html!(<div class="block-l">{for entries}</div>)
        });

        let mut rem = 0;

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

            let onclick = ctx.link().callback(move |_: MouseEvent| match event {
                Some(i) => Msg::Analyze(i),
                None => Msg::AnalyzeCycle,
            });

            let class = classes! {
                (rem > 0).then_some("active"),
                (!(event.is_none() && self.query.a.len() <= 1)).then_some("clickable"),
                "analyze-span"
            };

            rem = rem.saturating_sub(1);
            html!(<span {class} {onclick}>{c}</span>)
        });

        let analyze_hint = (self.query.a.len() > 1).then(|| {
            html!(
                <div class="block">{format!("{} / {}", self.query.i + 1, self.query.a.len())}</div>
            )
        });

        html! {
            <BrowserRouter>
                <div id="container">
                    <div class="block-l" id="prompt">
                        <input value={self.query.q.clone()} type="text" oninput={oninput} />
                    </div>

                    <div class="block" id="analyze">{for query}</div>
                    {for analyze_hint}

                    <>
                        {for entries}
                    </>
                </div>
            </BrowserRouter>
        }
    }
}

fn process_query(input: &str) -> String {
    let mut out = String::new();

    for segment in romaji::analyze(input) {
        out.push_str(segment.hiragana());
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
