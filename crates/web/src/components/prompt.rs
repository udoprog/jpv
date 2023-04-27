use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use lib::database::IndexExtra;
use lib::elements::{Entry, EntryKey};
use lib::romaji;
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlInputElement, Range};
use yew::prelude::*;
use yew_router::{prelude::*, AnyRoute};

use crate::components as c;

pub(crate) enum Msg {
    Change(String),
    Analyze(Range),
    HistoryChanged(Location),
}

#[derive(Default, Debug)]
struct Query {
    q: String,
    a: BTreeSet<String>,
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
                    this.a.insert(value);
                }
                _ => {
                }
            }
        }

        this
    }

    fn serialize(&self) -> Vec<(&'static str, &str)> {
        let mut out = Vec::new();
        out.push(("q", self.q.as_str()));

        for a in &self.a {
            out.push(("a", a.as_str()));
        }

        out
    }
}

#[derive(Default)]
pub(crate) struct Prompt {
    query: Query,
    entries: Vec<(BTreeSet<IndexExtra>, EntryKey, Entry<'static>)>,
    _handle: Option<LocationHandle>,
}

impl Prompt {
    fn refresh(&mut self, ctx: &Context<Self>, inputs: &BTreeSet<String>) {
        self.entries.clear();
        let mut dedup = HashMap::new();

        for input in inputs {
            for id in ctx.props().db.lookup(input) {
                let Ok(entry) = ctx.props().db.get(id) else {
                    continue;
                };

                let Some(&i) = dedup.get(&id.index()) else {
                    dedup.insert(id.index(), self.entries.len());
                    self.entries.push(([id.extra()].into_iter().collect(), EntryKey::default(), entry));
                    continue;
                };

                let Some((extras, _, _)) = self.entries.get_mut(i) else {
                    continue;
                };

                extras.insert(id.extra());
            }
        }

        for (id, key, e) in &mut self.entries {
            let conjugation = id.iter().any(|index| match index {
                IndexExtra::VerbInflection(..) => true,
                IndexExtra::AdjectiveInflection(..) => true,
                _ => false,
            });

            *key = e.sort_key(&inputs, conjugation);
        }

        self.entries.sort_by(|a, b| a.1.cmp(&b.1));
    }

    fn analyze(&mut self, ctx: &Context<Self>, range: &Range) -> Result<BTreeSet<String>> {
        fn error(error: JsValue) -> anyhow::Error {
            if let Some(string) = error.as_string() {
                anyhow!("{}", string)
            } else {
                anyhow!("an error occured")
            }
        }

        let node = range.common_ancestor_container().map_err(error)?;
        let mut inputs = BTreeSet::new();
        let mut longest = None;
        let original_end = range.end_offset().map_err(error)?;

        loop {
            let end = range.end_offset().map_err(error)?;

            let Ok(()) = range.set_end(&node, end + 1) else {
                break;
            };

            let Some(string) = range.to_string().as_string() else {
                continue;
            };

            if ctx.props().db.contains(&string) {
                inputs.insert(string);
                longest = Some(end + 1);
            }
        }

        if let Some(end) = longest {
            let _ = range.set_end(&node, end);
        } else {
            let _ = range.set_end(&node, original_end);
        }

        Ok(inputs)
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
                let inputs = [input.clone()].into_iter().collect();
                self.refresh(ctx, &inputs);

                if self.query.q != input {
                    self.query.q = input;
                    self.query.a.clear();
                    self.save_query(ctx, false);
                }

                true
            }
            Msg::Analyze(range) => {
                match self.analyze(ctx, &range) {
                    Err(error) => {
                        log::error!("Failed to analyze: {error}");
                    }
                    Ok(analyze) if !analyze.is_empty() => {
                        self.refresh(ctx, &analyze);

                        if self.query.a != analyze {
                            self.query.a = analyze;
                            self.save_query(ctx, true);
                        }
                    }
                    Ok(..) => {}
                }

                true
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

        let onclick = ctx.link().batch_callback(|_: MouseEvent| {
            let window: web_sys::Window = window()?;
            let selection: web_sys::Selection = window.get_selection().ok()??;
            let range = selection.get_range_at(0).ok()?;
            Some(Msg::Analyze(range))
        });

        let entries = (!self.entries.is_empty()).then(|| {
            let entries = self.entries.iter().map(|(extras, entry_key, entry)| {
                html!(<c::Entry extras={extras.clone()} entry_key={entry_key.clone()} entry={entry.clone()} />)
            });

            html!(<div class="block-l">{for entries}</div>)
        });

        html! {
            <BrowserRouter>
                <div id="container">
                    <div class="block-l" id="prompt">
                        <input value={self.query.q.clone()} type="text" oninput={oninput} />
                    </div>

                    <div class="block-1" id="analyze" {onclick}>
                        {self.query.q.clone()}
                    </div>

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

fn decode_query(location: Option<Location>) -> (Query, BTreeSet<String>) {
    let query = match location {
        Some(location) => location.query().ok(),
        None => None,
    };

    let query = query.unwrap_or_default();
    let query = Query::deserialize(query);

    let inputs = if query.a.is_empty() {
        [query.q.clone()].into_iter().collect()
    } else {
        query.a.clone()
    };

    (query, inputs)
}
