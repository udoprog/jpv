use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use lib::database::IndexExtra;
use lib::elements::{Entry, Example, KanjiElement, ReadingElement, Sense};
use web_sys::HtmlInputElement;
use yew::prelude::*;

enum Msg {
    Change(String),
}

#[derive(Default)]
struct App {
    value: String,
    entries: Vec<(BTreeSet<IndexExtra>, Entry<'static>)>,
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
            Msg::Change(value) => {
                self.entries.clear();

                let mut dedup = HashMap::new();

                for id in ctx.props().db.lookup(value.as_str()) {
                    let Ok(entry) = ctx.props().db.get(id) else {
                        continue;
                    };

                    let Some(&i) = dedup.get(&id.index()) else {
                        dedup.insert(id.index(), self.entries.len());
                        self.entries.push(([id.extra()].into_iter().collect(), entry));
                        continue;
                    };

                    let Some((extras, _)) = self.entries.get_mut(i) else {
                        continue;
                    };

                    extras.insert(id.extra());
                }

                self.value = value;
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
            let entries = self.entries.iter().map(render_entry);
            html!(<div class="block-x">{for entries}</div>)
        });

        html! {
            <div id="container">
                <div class="block-x">
                    <input id="prompt" value={self.value.clone()} type="text" oninput={oninput} />
                </div>

                <>
                    {for entries}
                </>
            </div>
        }
    }
}

macro_rules! bullets {
    ($base:ident . $name:ident) => {
        $base.$name.iter().map(|d| {
            html!(<span class={format!(concat!("bullet {name} {name}-{}"), d.ident(), name = stringify!($name))} title={d.help()}>{d.ident()}</span>)
        })
    }
}

fn render_entry((extras, entry): &(BTreeSet<IndexExtra>, Entry<'_>)) -> Html {
    let extras = extras.iter().flat_map(render_extra);

    let reading = render_seq(entry.reading_elements.iter(), render_reading);
    let kanji = render_seq(entry.kanji_elements.iter(), render_kanji);
    let senses = entry.senses.iter().enumerate().map(render_sense);

    html! {
        <div class="block-x entry">
            <div class="block-x sequence">{entry.sequence}</div>
            {for extras}
            <div class="block-x reading">{for reading}</div>
            <div class="block-x kanji">{for kanji}</div>
            <ul class="block-x senses">{for senses}</ul>
        </div>
    }
}

fn render_extra(extra: &IndexExtra) -> Option<Html> {
    let extra = match extra {
        IndexExtra::VerbInflection(inflection) => {
            format!("Verb conjugation: {inflection:?}")
        }
        IndexExtra::AdjectiveInflection(inflection) => {
            format!("Adjective inflection: {inflection:?}")
        }
        _ => return None,
    };

    Some(html!(<div class="block extra">{extra}</div>))
}

fn render_sense((_, sense): (usize, &Sense<'_>)) -> Html {
    let glossary = render_texts(sense.gloss.iter().map(|gloss| gloss.text));
    let examples = render_seq(sense.examples.iter(), render_example);

    let pos = bullets!(sense.pos);
    let misc = bullets!(sense.misc);
    let dialect = bullets!(sense.dialect);
    let field = bullets!(sense.field);

    html! {
        <li class="block block-x sense">
            <div>
                {for pos}
                {for misc}
                {for dialect}
                {for field}
            </div>
            <div class="block glossary">{for glossary}</div>
            <div class="block examples">{for examples}</div>
        </li>
    }
}

fn render_text(text: &str, last: bool) -> Html {
    let sep = (!last).then(|| html!(<span class="sep">{","}</span>));

    html! {
        <>
            <span class="text">{text}</span>
            {for sep}
        </>
    }
}

fn render_kanji(reading: &KanjiElement<'_>, last: bool) -> Html {
    let sep = (!last).then(|| html!(<span class="sep">{","}</span>));

    let priority = reading.priority.iter().map(|p| {
        html!(<span class={format!("bullet prio-{}", p.category())}>{p.category()}{p.level()}</span>)
    });

    let info = bullets!(reading.info);

    html! {
        <>
            <span class="text">{reading.text}</span>
            <span class="priority">{for priority}</span>
            <span class="info">{for info}</span>
            {for sep}
        </>
    }
}

fn render_reading(reading: &ReadingElement<'_>, last: bool) -> Html {
    let sep = (!last).then(|| html!(<span class="sep">{","}</span>));

    let priority = reading.priority.iter().map(|p| {
        html!(<span class={format!("bullet prio-{}", p.category())}>{p.category()}{p.level()}</span>)
    });

    let info = bullets!(reading.info);

    html! {
        <>
            <span class="text">{reading.text}</span>
            <span class="priority">{for priority}</span>
            <span class="info">{for info}</span>
            {for sep}
        </>
    }
}

fn render_example(example: &Example<'_>, _: bool) -> Html {
    let texts = render_texts(example.texts.iter().copied());

    let sent = example
        .sent
        .iter()
        .map(|sent| html!(<div class="block example-sentence">{sent.text}</div>));

    html! {
        <div class="example">
            <div class="block example-texts">{for texts}</div>
            {for sent}
        </div>
    }
}

fn render_seq<'a, I, T, B>(iter: I, builder: B) -> impl Iterator<Item = Html> + 'a
where
    I: IntoIterator<Item = &'a T>,
    I::IntoIter: 'a + DoubleEndedIterator,
    B: 'a + Copy + Fn(&T, bool) -> Html,
    T: ?Sized + 'a,
{
    let mut it = iter.into_iter();
    let last = it.next_back().map(move |value| builder(value, true));
    it.map(move |value| builder(value, false)).chain(last)
}

#[inline]
fn render_texts<'a, I>(iter: I) -> impl Iterator<Item = Html> + 'a
where
    I: IntoIterator<Item = &'a str>,
    I::IntoIter: 'a + DoubleEndedIterator,
{
    render_seq(iter, render_text)
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
