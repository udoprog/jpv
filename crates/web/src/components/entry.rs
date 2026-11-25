use std::collections::BTreeSet;

use lib::database::Source;
use lib::entities::KanjiInfo;
use lib::jmdict::{
    OwnedExample, OwnedExampleSentence, OwnedKanjiElement, OwnedReadingElement, OwnedSense,
};
use lib::{inflection, jmdict, Form, Furigana, Inflection, OwnedInflections, Priority};
use phrasebook::yew023::PhrasesContext;
use phrasebook::{args, Phrases};
use yew::prelude::*;

use sockel::Gap;

use super::{colon, comma, copy_text, icons, iter, romaji, row, ruby, seq, spacing, tag, tags};

use crate::class;
use crate::l10n::GLOSS_DEFAULT;

pub(crate) enum Msg {
    ToggleForm(usize, Form),
    ResetForm(usize),
    Change(String, Option<String>),
    AddTag(&'static str),
    AddPriority(Priority),
    Phrases(Phrases),
}

#[derive(Default)]
struct ExtraState {
    // Filter inflections to use among the select inflections.
    filter: Inflection,
}

#[derive(Debug)]
struct Combined {
    kanji: OwnedKanjiElement,
    reading: OwnedReadingElement,
}

impl Combined {
    fn is_common(&self) -> bool {
        !self.is_search_only() && !self.is_irregular() && !self.is_rare() && !self.is_outdated()
    }

    fn is_irregular(&self) -> bool {
        self.kanji.info.contains(KanjiInfo::IrregularKanji)
    }

    fn is_rare(&self) -> bool {
        self.kanji.info.contains(KanjiInfo::RareKanji)
    }

    fn is_outdated(&self) -> bool {
        self.kanji.info.contains(KanjiInfo::OutdatedKanji)
    }

    fn is_search_only(&self) -> bool {
        self.kanji.info.contains(KanjiInfo::SearchOnlyKanji)
    }

    /// Provide furigana iterator for the combined reading.
    fn furigana(&self) -> Furigana<'_> {
        Furigana::new(&self.kanji.text, &self.reading.text, "")
    }

    /// Test if this contains the given text.
    fn is_kanji(&self, text: &str) -> bool {
        self.kanji.text == text
    }
}

pub(crate) struct Entry {
    combined: Vec<Combined>,
    readings: Vec<OwnedReadingElement>,
    states: Vec<ExtraState>,
    inflections: Vec<(inflection::Reading, OwnedInflections)>,
    l10n: PhrasesContext,
}

#[derive(Properties)]
pub struct Props {
    pub embed: bool,
    pub sources: BTreeSet<Source>,
    pub entry: jmdict::OwnedEntry,
    /// The gloss language the entry was searched in, a JMdict ISO 639-2
    /// code such as `ger`.
    pub gloss: &'static str,
    pub onchange: Callback<(String, Option<String>), ()>,
    pub ontag: Callback<&'static str>,
    pub onpriority: Callback<Priority>,
}

impl PartialEq for Props {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.sources == other.sources
            && self.entry.sequence == other.entry.sequence
            && self.gloss == other.gloss
    }
}

impl Component for Entry {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let entry = borrowme::borrow(&ctx.props().entry);

        let mut this = Self {
            combined: Vec::new(),
            readings: Vec::new(),
            states: ctx
                .props()
                .sources
                .iter()
                .map(|_| ExtraState::default())
                .collect(),
            inflections: inflection::conjugate(&entry)
                .into_iter()
                .map(|(r, i, _)| (r, borrowme::to_owned(i)))
                .collect(),
            l10n: PhrasesContext::new(ctx.link(), ctx.link().callback(Msg::Phrases)),
        };

        this.refresh_entry(ctx);
        this
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleForm(index, form) => {
                if let Some(state) = self.states.get_mut(index) {
                    state.filter.toggle(form);
                }
            }
            Msg::ResetForm(index) => {
                if let Some(state) = self.states.get_mut(index) {
                    state.filter = Inflection::default();
                }
            }
            Msg::Change(text, english) => {
                ctx.props().onchange.emit((text, english));
            }
            Msg::AddTag(tag) => {
                ctx.props().ontag.emit(tag);
            }
            Msg::AddPriority(tag) => {
                ctx.props().onpriority.emit(tag);
            }
            Msg::Phrases(phrases) => {
                return self.l10n.set(phrases);
            }
        }

        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _: &Self::Properties) -> bool {
        let entry = borrowme::borrow(&ctx.props().entry);

        self.inflections = inflection::conjugate(&entry)
            .into_iter()
            .map(|(r, i, _)| (r, borrowme::to_owned(i)))
            .collect();

        self.states = ctx
            .props()
            .sources
            .iter()
            .map(|_| ExtraState::default())
            .collect();

        self.refresh_entry(ctx);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let phrases = self.l10n.phrases();
        let sources = &ctx.props().sources;
        let entry = &ctx.props().entry;

        let inflections =
            sources
                .iter()
                .zip(&self.states)
                .enumerate()
                .flat_map(|(index, (source, state))| {
                    Some((index, state, find_inflection(source, &self.inflections)?))
                });

        let extras =
            inflections
                .clone()
                .take(1)
                .flat_map(|(index, state, (inflection, inflections))| {
                    render_extra(ctx, phrases, index, inflection, inflections, state.filter)
                });

        let reading = iter(
            seq(
                self.readings.iter().filter(|r| !r.is_search_only()),
                |e, not_last| render_reading(ctx, e, not_last),
            ),
            |iter| row(html!({for iter})),
        );

        let common = iter(
            seq(
                self.combined.iter().filter(|c| c.is_common()),
                |e, not_last| render_combined(ctx, e, not_last),
            ),
            |iter| row(html!({for iter})),
        );

        let other_kana = iter(
            seq(
                self.readings.iter().filter(|c| c.is_search_only()),
                |e, not_last| render_reading(ctx, e, not_last),
            ),
            |iter| {
                row(
                    html!(<><span>{phrases.text("entry-other-kana")}</span>{colon()}{spacing()}{for iter}</>),
                )
            },
        );

        let other_kanji = iter(
            seq(
                self.combined.iter().filter(|c| !c.is_common()),
                |e, not_last| render_combined(ctx, e, not_last),
            ),
            |iter| {
                row(
                    html!(<><span>{phrases.text("entry-other-kanji")}</span>{colon()}{spacing()}{for iter}</>),
                )
            },
        );

        // Senses in the chosen gloss language come first, and the English
        // ones, which carry the tags and examples, are set apart below them.
        let gloss = ctx.props().gloss;
        let chosen = gloss != GLOSS_DEFAULT && entry.senses.iter().any(|s| is_lang(s, gloss));

        let (main, english): (Vec<_>, Vec<_>) = if chosen {
            entry.senses.iter().partition(|s| is_lang(s, gloss))
        } else {
            (entry.senses.iter().collect(), Vec::new())
        };

        let senses = iter(
            main.into_iter().map(|s| self.render_sense(ctx, s)),
            |iter| html!(<ol class={class::LIST_NUMERICAL}>{for iter}</ol>),
        );

        let english = iter(
            english.into_iter().map(|s| self.render_sense(ctx, s)),
            |iter| {
                html! {
                    <sockel::Stack gap={Gap::Snug}>
                        <span class={class::HINT}>{phrases.text("entry-english")}</span>
                        <ol class={classes!(class::LIST_NUMERICAL, class::SECONDARY)}>{for iter}</ol>
                    </sockel::Stack>
                }
            },
        );

        let actions = self.render_actions(phrases, entry);

        html! {
            <sockel::Stack gap={Gap::Tight}>
                {actions}
                {for extras}
                {for reading}
                {for common}
                {for senses}
                {for english}
                {for other_kana}
                {for other_kanji}
            </sockel::Stack>
        }
    }
}

impl Entry {
    fn refresh_entry(&mut self, ctx: &Context<Self>) {
        self.combined.clear();
        self.readings.clear();

        let entry = &ctx.props().entry;

        if entry.kanji_elements.is_empty() {
            self.readings.extend(entry.reading_elements.iter().cloned());
        } else {
            self.combined
                .extend(entry.kanji_elements.iter().flat_map(|kanji| {
                    entry.reading_elements.iter().flat_map(move |reading| {
                        reading.applies_to(&kanji.text).then_some(Combined {
                            kanji: kanji.clone(),
                            reading: reading.clone(),
                        })
                    })
                }));

            self.readings.extend(
                entry
                    .reading_elements
                    .iter()
                    .filter(|r| r.applies_to_nothing())
                    .cloned(),
            );

            if !self.combined.iter().any(|c| c.is_common()) {
                self.readings.extend(
                    entry
                        .reading_elements
                        .iter()
                        .filter(|r| !r.applies_to_nothing())
                        .cloned(),
                );
            }
        }
    }

    /// The entry's menu: copying its word or reading, and its data.
    fn render_actions(&self, phrases: &Phrases, entry: &jmdict::OwnedEntry) -> Html {
        let word = self
            .combined
            .iter()
            .find(|c| c.is_common())
            .map(|c| c.kanji.text.clone());

        let reading = entry.reading_elements.first().map(|r| r.text.clone());

        let copy = |text: Option<String>, label: std::rc::Rc<str>| {
            let disabled = text.is_none();
            let onclick = Callback::from(move |_: MouseEvent| {
                if let Some(text) = &text {
                    copy_text(text);
                }
            });

            html! {
                <sockel::MenuItem icon={icons::glyph(sockel::Glyph::Copy)} {disabled} {onclick}>{label}</sockel::MenuItem>
            }
        };

        let copy_word = copy(word.or_else(|| reading.clone()), phrases.text("entry-copy"));
        let copy_reading = copy(reading, phrases.text("entry-copy-reading"));
        let detail = phrases.msg(
            "entry-open-data-detail",
            &args!(sequence = entry.sequence.to_string()),
        );

        html! {
            <sockel::Cluster justify={sockel::Justify::End}>
                <sockel::Menu variant={sockel::ButtonVariant::Quiet} size={sockel::ButtonSize::Small}
                    label={phrases.text("entry-actions")} tip={phrases.text("entry-actions")} width={sockel::MenuWidth::Wide}>
                    {copy_word}
                    {copy_reading}
                    <sockel::MenuSeparator />
                    <sockel::MenuItem icon={icons::named(icons::DATA)} href={format!("/api/entry/{}", entry.sequence)} new_tab={true} {detail}>
                        {phrases.text("entry-open-data")}
                    </sockel::MenuItem>
                </sockel::Menu>
            </sockel::Cluster>
        }
    }

    fn render_sense(&self, ctx: &Context<Self>, s: &OwnedSense) -> Html {
        let phrases = self.l10n.phrases();
        let info = s.info.as_ref().map(|info| row(html!({ info })));

        let stags = seq(s.stagr.iter().chain(s.stagk.iter()), |text, not_last| {
            let stag = if let Some(c) = self.combined.iter().find(|c| c.is_kanji(text)) {
                html!(<span data-tip={romaji(c.furigana())}>{ruby(c.furigana())}</span>)
            } else {
                html!(<>{text}</>)
            };

            let onclick = ctx.link().callback({
                let text = text.to_owned();
                move |_: MouseEvent| Msg::Change(text.clone(), None)
            });

            html!(<><span class={class::CLICKABLE} {onclick}>{stag}{for not_last.then(comma)}</span></>)
        });

        let stag = iter(stags, |stags| {
            row(
                html!(<><span>{phrases.text("entry-applies-to")}</span>{colon()}{spacing()}{for stags}</>),
            )
        });

        let glossary = texts(s.gloss.iter().map(|gloss| &gloss.text), None);
        let bullets = tags(
            bullets!(ctx, s.pos)
                .chain(bullets!(ctx, s.misc))
                .chain(bullets!(ctx, s.dialect))
                .chain(bullets!(ctx, s.field)),
        );

        let glossary = iter(glossary.chain(bullets), |iter| row(html!({for iter})));

        let examples = iter(
            s.examples.iter().map(|e| self.render_example(ctx, e)),
            |iter| html!(<sockel::Stack gap={Gap::Snug}>{for iter}</sockel::Stack>),
        );

        html! {
            <li>
                <sockel::Stack gap={Gap::Snug}>
                    {for glossary}
                    {for info}
                    {for stag}
                    {for examples}
                </sockel::Stack>
            </li>
        }
    }

    fn render_example(&self, ctx: &Context<Self>, example: &OwnedExample) -> Html {
        struct Languages<'a> {
            jpn: &'a str,
            eng: Option<&'a str>,
        }

        fn languages(sentences: &[OwnedExampleSentence]) -> Option<Languages<'_>> {
            let mut jpn = None;
            let mut eng = None;

            for sent in sentences {
                let out = match sent.lang.as_deref() {
                    Some("eng") => &mut eng,
                    Some("jpn") | None => &mut jpn,
                    _ => continue,
                };

                *out = Some(sent.text.as_str());
            }

            Some(Languages { jpn: jpn?, eng })
        }

        let texts = seq(example.texts.iter(), |text, not_last| {
            let onclick = ctx.link().callback({
                let text = text.to_owned();
                move |_: MouseEvent| Msg::Change(text.clone(), None)
            });

            let text = if let Some(c) = self.combined.iter().find(|c| c.is_kanji(text)) {
                html!(<span data-tip={romaji(c.furigana())}>{ruby(c.furigana())}</span>)
            } else {
                html!(<>{text}</>)
            };

            html!(<><span class={classes!(class::TEXT, class::CLICKABLE)} {onclick}>{text}</span>{for not_last.then(comma)}</>)
        });

        let sent = languages(&example.sentences).map(|l: Languages<'_>| {
            let onclick = ctx.link().callback({
                let jpn = l.jpn.to_owned();
                let eng = l.eng.map(ToOwned::to_owned);
                move |_: MouseEvent| Msg::Change(jpn.clone(), eng.clone())
            });

            let eng = l.eng.map(|text| html!(<>{spacing()}<span>{text}</span></>));

            html!(<>{colon()}{spacing()}<span class={class::CLICKABLE} {onclick}>{l.jpn}</span>{for eng}</>)
        });

        html! {
            row(html!(<>{for texts}{for sent}</>))
        }
    }
}

/// Find the matching inflection based on the source.
fn find_inflection<'a>(
    source: &Source,
    inflections: &'a [(inflection::Reading, OwnedInflections)],
) -> Option<(Inflection, &'a OwnedInflections)> {
    match source {
        Source::Inflection { data } => {
            let (_, inflections) = inflections.iter().find(|(r, _)| *r == data.reading)?;
            Some((data.inflection, inflections))
        }
        _ => None,
    }
}

fn render_extra(
    ctx: &Context<Entry>,
    phrases: &Phrases,
    index: usize,
    inflection: Inflection,
    inflections: &OwnedInflections,
    filter: Inflection,
) -> Option<Html> {
    let word = inflections.get(inflection ^ filter);

    let word = word.map(
        |w| row(html!(<span class={classes!(class::TEXT, class::KANJI)} data-tip={romaji(w.furigana())}>{ruby(w.furigana())}</span>)),
    );

    let inflection_html = render_inflection(ctx, phrases, index, inflection, filter, inflections);
    let tutorials = render_tutorials(phrases, inflection, filter);

    Some(html! {
        <sockel::Card inset={sockel::Inset::Compact} gap={Gap::Tight} class={class::INFLECTION_NOTICE}>
            <div class={class::BLOCK_TITLE}>{phrases.text("entry-inflection")}</div>
            <sockel::Cluster gap={Gap::Snug}>{for inflection_html}</sockel::Cluster>
            {tutorials}
            {for word}
        </sockel::Card>
    })
}

fn render_inflection<'a>(
    ctx: &'a Context<Entry>,
    phrases: &Phrases,
    index: usize,
    inflection: Inflection,
    filter: Inflection,
    inflections: &'a OwnedInflections,
) -> impl Iterator<Item = Html> + 'a {
    let this = filter ^ inflection;

    let form = Inflection::all().iter().flat_map(move |f| {
        let mut candidate = this;
        candidate.toggle(f);

        let exists = inflections.contains(candidate);

        if !exists && !this.contains(f) {
            return None;
        }

        let on_toggle = ctx.link().callback(move |_: bool| Msg::ToggleForm(index, f));

        Some(html! {
            <span data-tip={f.title()}>
                <sockel::ChipToggle on={this.contains(f)} disabled={!exists} {on_toggle}>{f.describe()}</sockel::ChipToggle>
            </span>
        })
    });

    let onclick = ctx
        .link()
        .callback(move |_: MouseEvent| Msg::ResetForm(index));

    let reset = (!filter.is_empty()).then(|| {
        let text = phrases.text("entry-reset");
        html!(<sockel::Button size={sockel::ButtonSize::Small} variant={sockel::ButtonVariant::Danger} icon={icons::named(icons::RESET)} {onclick}>{text}</sockel::Button>)
    });

    form.chain(reset)
}

fn render_tutorials(phrases: &Phrases, inflection: Inflection, filter: Inflection) -> Html {
    let this = filter ^ inflection;

    let mut tutorials = Inflection::all().iter().flat_map(|f| {
        if this.contains(f) {
            Some((f, f.url()?))
        } else {
            None
        }
    });

    let first = tutorials.next();

    let Some(first) = first else {
        return html!();
    };

    let tutorials = seq(
        [first].into_iter().chain(tutorials),
        |(f, url), not_last| {
            html! {
                <>
                    <a href={url} target="_tutorial" data-tip={phrases.msg("entry-tutorial-tip", &args!(form = f.title()))}>{phrases.msg("entry-tutorial", &args!(form = f.describe()))}</a>
                    {for not_last.then(comma)}
                </>
            }
        },
    );

    html!(<sockel::Cluster gap={Gap::None} class={class::TUTORIALS}>{for tutorials}</sockel::Cluster>)
}

fn render_reading(ctx: &Context<Entry>, reading: &OwnedReadingElement, not_last: bool) -> Html {
    let priority = reading.priority.iter().map(|p| render_priority(ctx, p));

    let bullets = tags(priority.chain(bullets!(ctx, reading.info)));

    let onclick = ctx.link().callback({
        let text = reading.text.to_owned();
        move |_: MouseEvent| Msg::Change(text.clone(), None)
    });

    html! {
        <>
            <a class={classes!(class::TEXT, class::KANJI)} {onclick}>{&reading.text}</a>
            {for bullets}
            {for not_last.then(comma)}
        </>
    }
}

fn render_combined(
    ctx: &Context<Entry>,
    c @ Combined { kanji, .. }: &Combined,
    not_last: bool,
) -> Html {
    let priority = kanji.priority.iter().map(|p| render_priority(ctx, p));

    let bullets = tags(priority.chain(bullets!(ctx, kanji.info)));

    let onclick = ctx.link().callback({
        let text = c.kanji.text.to_owned();
        move |_: MouseEvent| Msg::Change(text.clone(), None)
    });

    html! {
        <>
            <a class={classes!(class::TEXT, class::KANJI)} {onclick} data-tip={romaji(c.furigana())}>{ruby(c.furigana())}</a>
            {for bullets}
            {for not_last.then(comma)}
        </>
    }
}

fn render_priority(ctx: &Context<Entry>, p: &Priority) -> Html {
    let onclick = ctx.link().callback({
        let p = *p;
        move |_: MouseEvent| Msg::AddPriority(p)
    });

    tag(format!("{}{}", p.category(), p.level()), p.title(), onclick)
}

/// A simple text sequence renderer.
#[inline]
fn texts<'a, I>(iter: I, extra: Option<&'static str>) -> impl Iterator<Item = Html> + 'a
where
    I: IntoIterator,
    I::Item: 'a + AsRef<str>,
    I::IntoIter: 'a + DoubleEndedIterator,
{
    seq(iter, move |text, not_last| {
        let class = classes!(class::TEXT, extra);

        html! {
            <>
                <span class={class}>{text.as_ref()}</span>
                {for not_last.then(comma)}
            </>
        }
    })
}

/// Test if any of the sense's glosses are in `lang`, English when a gloss
/// names none.
fn is_lang(sense: &OwnedSense, lang: &str) -> bool {
    sense
        .gloss
        .iter()
        .any(|g| g.lang.as_deref().unwrap_or(GLOSS_DEFAULT) == lang)
}
