use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use lib::database::IndexExtra;
use lib::elements::{Entry, EntryKey, Example, KanjiElement, ReadingElement, Sense};
use lib::entities::KanjiInfo;
use lib::{kana, romaji};
use web_sys::HtmlInputElement;
use yew::prelude::*;

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
            let entries = self.entries.iter().map(render_entry);
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

macro_rules! bullets {
    ($base:ident . $name:ident) => {
        $base.$name.iter().map(|d| {
            html!(<span class={format!(concat!("bullet {name} {name}-{}"), d.ident(), name = stringify!($name))} title={d.help()}>{d.ident()}</span>)
        })
    }
}

fn render_entry((extras, key, entry): &(BTreeSet<IndexExtra>, EntryKey, Entry<'_>)) -> Html {
    let extras = extras.iter().flat_map(render_extra);

    let (reading, combined) = if entry.kanji_elements.is_empty() {
        let reading = render_seq(entry.reading_elements.iter(), render_reading);

        let reading = (!entry.reading_elements.is_empty())
            .then(|| html!(<div class="block-l entry-reading">{for reading}</div>));

        (reading, None)
    } else {
        let iter = entry.kanji_elements.iter().flat_map(|kanji| {
            entry
                .reading_elements
                .iter()
                .flat_map(move |reading| reading.applies_to(kanji.text).then_some((reading, kanji)))
        });

        let combined = render_seq(iter, render_combined);

        let combined = html! {
            html!(<div class="block-l entry-kanji">{for combined}</div>)
        };

        let all_kanjis_rare = entry
            .kanji_elements
            .iter()
            .all(|k| k.info.contains(KanjiInfo::RareKanji));

        let mut all_readings = entry.reading_elements.iter();
        let mut filtered;

        let reading: &mut dyn DoubleEndedIterator<Item = _> = if !all_kanjis_rare {
            filtered = all_readings.filter(|r| r.no_kanji);
            &mut filtered
        } else {
            // Render regular readings, because *all* kanji readings are rare.
            &mut all_readings
        };

        let reading = render_seq(reading, render_reading);

        let reading = (!entry.reading_elements.is_empty())
            .then(|| html!(<div class="block-l entry-reading">{for reading}</div>));

        (reading, Some(combined))
    };

    let senses = entry.senses.iter().enumerate().map(render_sense);

    html! {
        <div class="block-l entry">
            <div class="block-l entry-sequence">{entry.sequence}</div>
            <div class="block-l entry-key">{format!("{:?}", key)}</div>
            {for extras}
            {for reading}
            {for combined}
            <ul class="block-l entry-senses">{for senses}</ul>
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

fn render_sense((_, s): (usize, &Sense<'_>)) -> Html {
    let any =
        !s.pos.is_empty() || !s.misc.is_empty() || !s.dialect.is_empty() || !s.field.is_empty();

    let bullets = any.then(|| {
        let pos = bullets!(s.pos);
        let misc = bullets!(s.misc);
        let dialect = bullets!(s.dialect);
        let field = bullets!(s.field);
        html!(<span class="entry-sense-bullets">{for pos}{for misc}{for dialect}{for field}</span>)
    });

    let glossary = render_texts(s.gloss.iter().map(|gloss| gloss.text));
    let glossary = (!s.gloss.is_empty())
        .then(move || html!(<div class="block entry-glossary">{for glossary}{for bullets}</div>));

    let examples = render_seq(s.examples.iter(), render_example);
    let examples = (!s.examples.is_empty())
        .then(move || html!(<div class="block entry-examples">{for examples}</div>));

    html! {
        <li class="block-l entry-sense">
            {for glossary}
            {for examples}
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

fn render_reading(reading: &ReadingElement<'_>, last: bool) -> Html {
    let sep = (!last).then(|| html!(<span class="sep">{","}</span>));

    let priority = (!reading.priority.is_empty()).then(move || {
        let priority = reading.priority.iter().map(|p| {
            html!(<span class={format!("bullet prio-{}", p.category())}>{p.category()}{p.level()}</span>)
        });

        html!(<span class="priority">{for priority}</span>)
    });

    let info = (!reading.info.is_empty()).then(|| {
        let info = bullets!(reading.info);
        html!(<span class="info">{for info}</span>)
    });

    let bullets = (!reading.priority.is_empty() || !reading.info.is_empty())
        .then(|| html!(<span class="entry-reading-bullets">{for priority}{for info}</span>));

    html! {
        <>
            <span class="text">{reading.text}</span>
            {for bullets}
            {for sep}
        </>
    }
}

fn render_combined((reading, kanji): (&ReadingElement<'_>, &KanjiElement<'_>), last: bool) -> Html {
    let sep = (!last).then(|| html!(<span class="sep">{","}</span>));

    let priority = (!kanji.priority.is_empty()).then(|| {
        let priority = kanji.priority.iter().map(|p| {
            html!(<span class={format!("bullet prio-{}", p.category())}>{p.category()}{p.level()}</span>)
        });

        html!(<span class="priority">{for priority}</span>)
    });

    let info = (!kanji.info.is_empty()).then(|| {
        let info = bullets!(kanji.info);
        html!(<span class="info">{for info}</span>)
    });

    let bullets = (!kanji.priority.is_empty() || !kanji.info.is_empty())
        .then(move || html!(<span class="entry-kanji-bullets">{for priority}{for info}</span>));

    let furigana = kana::Word::new(kanji.text, reading.text).furigana();

    let text = furigana.iter().map(|group| match group {
        lib::FuriganaGroup::Kanji(kanji, kana) => {
            html!(<>{kanji}<rt>{kana}</rt></>)
        }
        lib::FuriganaGroup::Kana(kana) => {
            html!({ kana })
        }
    });

    html! {
        <>
            <span class="text"><ruby>{for text}</ruby></span>
            {for bullets}
            {for sep}
        </>
    }
}

fn render_example(example: &Example<'_>, _: bool) -> Html {
    let texts = render_texts(example.texts.iter().copied());

    let sent = example
        .sent
        .iter()
        .map(|sent| html!(<span class="block entry-example-sentence">{sent.text}</span>));

    html! {
        <div class="entry-example">{for texts}<span class="sep">{":"}</span>{for sent}</div>
    }
}

fn render_seq<'a, I, T, B>(iter: I, builder: B) -> impl Iterator<Item = Html> + 'a
where
    I: IntoIterator<Item = T>,
    I::IntoIter: 'a + DoubleEndedIterator,
    B: 'a + Copy + Fn(T, bool) -> Html,
    T: 'a,
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
