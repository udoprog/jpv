mod components;

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use lib::database::IndexExtra;
use lib::elements::{Entry, EntryKey};
use lib::romaji;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use self::components as c;

enum Msg {
    Change(String),
}

#[derive(Default)]
struct App {
    value: String,
    entries: Vec<(BTreeSet<IndexExtra>, EntryKey, Entry<'static>)>,
}

#[derive(Properties)]
struct Props {
    db: Arc<lib::database::Database<'static>>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.db, &other.db)
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Self::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Change(input) => {
                self.entries.clear();

                let mut query = String::new();

                for segment in romaji::analyze(&input) {
                    query.push_str(segment.hiragana());
                }

                let mut dedup = HashMap::new();

                for id in ctx.props().db.lookup(query.as_str()) {
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

                for (id, key, e) in &mut self.entries {
                    let conjugation = id.iter().any(|index| match index {
                        IndexExtra::VerbInflection(..) => true,
                        IndexExtra::AdjectiveInflection(..) => true,
                        _ => false,
                    });

                    *key = e.sort_key(&query, conjugation);
                }

                self.entries.sort_by(|a, b| a.1.cmp(&b.1));
                self.value = query;
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
            let entries = self.entries.iter().map(|(extras, entry_key, entry)| {
                html!(<c::Entry extras={extras.clone()} entry_key={entry_key.clone()} entry={entry.clone()} />)
            });

            html!(<div class="block-l">{for entries}</div>)
        });

        html! {
            <div id="container">
                <div class="block-l">
                    <input id="prompt" value={self.value.clone()} type="text" oninput={oninput} />
                </div>

                <>
                    {for entries}
                </>
            </div>
        }
    }
}

const INDEX: &[u8] = include_bytes!("../../../index.bin");
const DATABASE: &[u8] = include_bytes!("../../../database.bin");

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    let index = lib::database::IndexRef::from_bytes(INDEX.as_ref()).expect("broken index");
    let db = Arc::new(lib::database::Database::new(DATABASE.as_ref(), index));

    log::info!("Started up");

    yew::Renderer::<App>::with_props(Props { db }).render();
}
