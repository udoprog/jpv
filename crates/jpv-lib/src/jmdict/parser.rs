use std::collections::HashSet;
use std::mem;

use anyhow::{anyhow, bail, Context, Result};
use fixed_map::Set;
use relative_path::RelativePathBuf;
use xmlparser::{ElementEnd, Token, Tokenizer};

use crate::entities::{Dialect, Field, KanjiInfo, Miscellaneous, ReadingInfo};
use crate::jmdict::Entry;
use crate::PartOfSpeech;
use crate::Priority;

use super::{
    Example, ExampleSentence, ExampleSource, Glossary, KanjiElement, ReadingElement, Sense,
    SourceLanguage,
};

enum State<'a> {
    Root,
    Entry(EntryBuilder<'a>),
    ReadingElement(ReadingElementBuilder<'a>),
    KanjiElement(KanjiElementBuilder<'a>),
    Sense(Sense<'a>),
    Gloss(GlossBuilder<'a>),
    Example(ExampleBuilder<'a>),
    ExampleSource(ExampleSourceBuilder<'a>),
    ExampleSentence(ExampleSentenceBuilder<'a>),
    SourceLanguage(SourceLanguageBuilder<'a>),
    Empty(&'a str),
    Text(&'a str, Option<&'a str>),
    U64(&'a str, Option<u64>),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Output<'a> {
    Text(&'a str),
    Open(&'a str),
    Attribute(&'a str, &'a str),
    Close,
    Eof,
}

pub struct Parser<'a> {
    closed: bool,
    path: RelativePathBuf,
    tokenizer: Tokenizer<'a>,
    stack: Vec<State<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            closed: false,
            path: RelativePathBuf::new(),
            tokenizer: Tokenizer::from(input),
            stack: Vec::new(),
        }
    }

    /// Parse the next entry.
    pub fn parse(&mut self) -> Result<Option<Entry<'a>>> {
        self.parse_inner()
            .with_context(|| self.path.as_str().to_string())
    }

    fn parse_inner(&mut self) -> Result<Option<Entry<'a>>> {
        loop {
            let output = self.parse_next()?;

            macro_rules! set_option {
                ($output:expr, $value:expr) => {
                    if $output.is_some() {
                        bail!("Duplicate value");
                    }

                    $output.replace($value);
                };
            }

            match output {
                Output::Text(value) => match &mut self.stack[..] {
                    [.., State::U64(_, text)] => {
                        set_option!(text, value.parse()?);
                    }
                    [.., State::Text(_, text)] => {
                        set_option!(text, value);
                    }
                    [.., State::Gloss(builder)] => {
                        set_option!(builder.text, value);
                    }
                    [.., State::ExampleSource(builder)] => {
                        set_option!(builder.text, value);
                    }
                    [.., State::ExampleSentence(builder)] => {
                        set_option!(builder.text, value);
                    }
                    [.., State::SourceLanguage(builder)] => {
                        set_option!(builder.text, value);
                    }
                    _ => {
                        bail!("Unexpected text: {value}");
                    }
                },
                Output::Open(element) => match (&self.stack[..], element) {
                    ([], "JMdict") => {
                        self.stack.push(State::Root);
                    }
                    ([State::Root], "entry") => {
                        self.stack.push(State::Entry(EntryBuilder::default()));
                    }
                    ([.., State::Entry(..)], "ent_seq") => {
                        self.stack.push(State::U64("ent_seq", None));
                    }
                    ([.., State::Entry(..)], "r_ele") => {
                        self.stack
                            .push(State::ReadingElement(ReadingElementBuilder::default()));
                    }
                    ([.., State::Entry(..)], "k_ele") => {
                        self.stack
                            .push(State::KanjiElement(KanjiElementBuilder::default()));
                    }
                    ([.., State::Entry(..)], "sense") => {
                        self.stack.push(State::Sense(Sense::default()));
                    }
                    (
                        [.., State::ReadingElement(..)],
                        element @ ("reb" | "re_restr" | "re_pri" | "re_inf"),
                    ) => {
                        self.stack.push(State::Text(element, None));
                    }
                    ([.., State::ReadingElement(..)], "re_nokanji") => {
                        self.stack.push(State::Empty("re_nokanji"));
                    }
                    ([.., State::KanjiElement(..)], element @ ("keb" | "ke_pri" | "ke_inf")) => {
                        self.stack.push(State::Text(element, None));
                    }
                    (
                        [.., State::Sense(..)],
                        element @ ("pos" | "xref" | "s_inf" | "dial" | "stagk" | "stagr" | "field"
                        | "misc" | "ant"),
                    ) => {
                        self.stack.push(State::Text(element, None));
                    }
                    ([.., State::Sense(..)], "gloss") => {
                        self.stack.push(State::Gloss(GlossBuilder::default()));
                    }
                    ([.., State::Sense(..)], "example") => {
                        self.stack.push(State::Example(ExampleBuilder::default()));
                    }
                    ([.., State::Sense(..)], "lsource") => {
                        self.stack
                            .push(State::SourceLanguage(SourceLanguageBuilder::default()));
                    }
                    ([.., State::Example(..)], "ex_srce") => {
                        self.stack
                            .push(State::ExampleSource(ExampleSourceBuilder::default()));
                    }
                    ([.., State::Example(..)], "ex_sent") => {
                        self.stack
                            .push(State::ExampleSentence(ExampleSentenceBuilder::default()));
                    }
                    ([.., State::Example(..)], "ex_text") => {
                        self.stack.push(State::Text("ex_text", None));
                    }
                    _ => {
                        bail!("Unexpected element: {element}");
                    }
                },
                Output::Attribute(key, value) => match (&mut self.stack[..], key) {
                    ([.., State::Gloss(builder)], "g_type") => {
                        set_option!(builder.ty, value);
                    }
                    ([.., State::ExampleSource(builder)], "exsrc_type") => {
                        set_option!(builder.ty, value);
                    }
                    ([.., State::ExampleSentence(builder)], "lang") => {
                        set_option!(builder.lang, value);
                    }
                    ([.., State::SourceLanguage(builder)], "lang") => {
                        set_option!(builder.lang, value);
                    }
                    ([.., State::SourceLanguage(builder)], "ls_type") => {
                        set_option!(builder.ty, value);
                    }
                    ([.., State::SourceLanguage(builder)], "ls_wasei") => {
                        let value = match value {
                            "y" => true,
                            other => bail!("Invalid attribute value: {other}"),
                        };

                        builder.waseigo = value;
                    }
                    _ => {
                        bail!("Unexpected attribute: {key}");
                    }
                },
                Output::Close => {
                    let top = self.stack.pop().context("Expected state")?;

                    macro_rules! entity {
                        ($out:ident.$field:ident: $ty:ty = $text:expr) => {
                            let text = $text.context("Missing text")?;
                            let $field = <$ty>::parse(text)
                                .with_context(|| anyhow!("Invalid entity: {text}"))?;
                            $out.$field.insert($field);
                        };
                    }

                    match (&mut self.stack[..], top) {
                        ([.., State::Entry(entry)], State::U64("ent_seq", sequence)) => {
                            let sequence = sequence.context("Missing entity sequence")?;
                            set_option!(entry.sequence, sequence);
                        }
                        ([.., State::ReadingElement(builder)], State::Text("reb", text)) => {
                            let text = text.context("Missing text")?;
                            set_option!(builder.text, text);
                        }
                        ([.., State::ReadingElement(builder)], State::Text("re_restr", text)) => {
                            builder.reading_string.insert(text.context("Missing text")?);
                        }
                        ([.., State::ReadingElement(builder)], State::Text("re_pri", text)) => {
                            let text = text.context("Missing text")?;
                            let priority = Priority::parse(text)
                                .with_context(|| anyhow!("Invalid priority: {text}"))?;
                            builder.priority.push(priority);
                        }
                        ([.., State::ReadingElement(builder)], State::Text("re_inf", text)) => {
                            entity!(builder.info: ReadingInfo = text);
                        }
                        ([.., State::ReadingElement(builder)], State::Empty("re_nokanji")) => {
                            builder.no_kanji = true;
                        }
                        ([.., State::KanjiElement(builder)], State::Text("keb", text)) => {
                            let text = text.context("Missing text")?;
                            set_option!(builder.text, text);
                        }
                        ([.., State::KanjiElement(builder)], State::Text("ke_pri", text)) => {
                            let text = text.context("Missing text")?;
                            let priority = Priority::parse(text)
                                .with_context(|| anyhow!("Invalid priority: {text}"))?;
                            builder.priority.push(priority);
                        }
                        ([.., State::KanjiElement(builder)], State::Text("ke_inf", text)) => {
                            entity!(builder.info: KanjiInfo = text);
                        }
                        ([.., State::Entry(entry)], State::ReadingElement(builder)) => {
                            let text = builder.text.context("Missing text")?;

                            entry.reading_elements.push(ReadingElement {
                                text,
                                no_kanji: builder.no_kanji,
                                reading_string: builder.reading_string,
                                priority: builder.priority,
                                info: builder.info,
                            });
                        }
                        ([.., State::Entry(entry)], State::KanjiElement(builder)) => {
                            entry.kanji_elements.push(KanjiElement {
                                text: builder.text.context("Missing text")?,
                                priority: builder.priority,
                                info: builder.info,
                            });
                        }
                        ([.., State::Sense(builder)], State::Text("pos", text)) => {
                            entity!(builder.pos: PartOfSpeech = text);
                        }
                        ([.., State::Sense(builder)], State::Text("xref", text)) => {
                            builder.xref.push(text.context("Missing xref")?);
                        }
                        ([.., State::Sense(builder)], State::Text("s_inf", text)) => {
                            set_option!(builder.info, text.context("Missing sense information")?);
                        }
                        ([.., State::Sense(builder)], State::Text("misc", text)) => {
                            entity!(builder.misc: Miscellaneous = text);
                        }
                        ([.., State::Sense(builder)], State::Text("ant", text)) => {
                            builder.antonym.push(text.context("Missing antonym")?);
                        }
                        ([.., State::Sense(builder)], State::Text("dial", text)) => {
                            entity!(builder.dialect: Dialect = text);
                        }
                        ([.., State::Sense(builder)], State::Text("field", text)) => {
                            entity!(builder.field: Field = text);
                        }
                        ([.., State::Sense(builder)], State::Text("stagk", text)) => {
                            builder.stagk.push(text.context("Missing stagk")?);
                        }
                        ([.., State::Sense(builder)], State::Text("stagr", text)) => {
                            builder.stagr.push(text.context("Missing stagr")?);
                        }
                        ([.., State::Sense(sense)], State::Gloss(builder)) => {
                            sense.gloss.push(Glossary {
                                text: builder.text.context("Missing glossary text")?,
                                ty: builder.ty,
                                lang: builder.lang,
                            });
                        }
                        ([.., State::Sense(sense)], State::Example(example)) => {
                            sense.examples.push(Example {
                                sentences: example.sentences,
                                sources: example.sources,
                                texts: example.texts,
                            });
                        }
                        ([.., State::Sense(sense)], State::SourceLanguage(source)) => {
                            sense.source_language.push(SourceLanguage {
                                text: source.text,
                                lang: source.lang,
                                waseigo: source.waseigo,
                                ty: source.ty,
                            });
                        }
                        ([.., State::Entry(entry)], State::Sense(sense)) => {
                            entry.senses.push(sense);
                        }
                        ([.., State::Example(example)], State::ExampleSource(source)) => {
                            example.sources.push(ExampleSource {
                                text: source.text.context("Missing source text")?,
                                ty: source.ty,
                            });
                        }
                        ([.., State::Example(example)], State::ExampleSentence(sentence)) => {
                            example.sentences.push(ExampleSentence {
                                text: sentence.text.context("Missing sentence text")?,
                                lang: sentence.lang,
                            });
                        }
                        ([.., State::Example(example)], State::Text("ex_text", text)) => {
                            example.texts.push(text.context("Missing example text")?);
                        }
                        ([State::Root], State::Entry(builder)) => {
                            return Ok(Some(Entry {
                                sequence: builder.sequence.context("Missing sequence")?,
                                reading_elements: builder.reading_elements,
                                kanji_elements: builder.kanji_elements,
                                senses: builder.senses,
                            }))
                        }
                        ([], State::Root) => {}
                        _ => {
                            bail!("Unexpected element close");
                        }
                    }
                }
                Output::Eof => {
                    if !self.stack.is_empty() {
                        bail!("Unexpected EOF");
                    }

                    return Ok(None);
                }
            }
        }
    }

    fn parse_next(&mut self) -> Result<Output<'a>> {
        loop {
            if mem::take(&mut self.closed) {
                self.path.pop();
            }

            let Some(token) = self.tokenizer.next() else {
                return Ok(Output::Eof);
            };

            match token? {
                Token::Text { text } => {
                    if text.as_str().trim().is_empty() {
                        continue;
                    }

                    return Ok(Output::Text(text.as_str()));
                }
                Token::Cdata { text, .. } => {
                    if text.as_str().trim().is_empty() {
                        continue;
                    }

                    return Ok(Output::Text(text.as_str()));
                }
                Token::ElementStart { local, .. } => {
                    self.path.push(local.as_str());
                    tracing::trace!(path = self.path.as_str(), "enter");
                    return Ok(Output::Open(local.as_str()));
                }
                Token::ElementEnd { end, .. } => {
                    if let ElementEnd::Close { .. } | ElementEnd::Empty { .. } = end {
                        tracing::trace!(path = self.path.as_str(), "leave");
                        self.closed = true;
                        return Ok(Output::Close);
                    }
                }
                Token::Attribute { local, value, .. } => {
                    return Ok(Output::Attribute(local.as_str(), value.as_str()));
                }
                _ => {
                    // intentionally ignore unsupported data.
                }
            }
        }
    }
}

#[derive(Default)]
struct EntryBuilder<'a> {
    sequence: Option<u64>,
    reading_elements: Vec<ReadingElement<'a>>,
    kanji_elements: Vec<KanjiElement<'a>>,
    senses: Vec<Sense<'a>>,
}

#[derive(Default)]
struct ReadingElementBuilder<'a> {
    text: Option<&'a str>,
    no_kanji: bool,
    reading_string: HashSet<&'a str>,
    priority: Vec<Priority>,
    info: Set<ReadingInfo>,
}

#[derive(Default)]
struct KanjiElementBuilder<'a> {
    text: Option<&'a str>,
    priority: Vec<Priority>,
    info: Set<KanjiInfo>,
}

#[derive(Default)]
struct GlossBuilder<'a> {
    text: Option<&'a str>,
    ty: Option<&'a str>,
    lang: Option<&'a str>,
}

#[derive(Default)]
struct ExampleBuilder<'a> {
    sentences: Vec<ExampleSentence<'a>>,
    sources: Vec<ExampleSource<'a>>,
    texts: Vec<&'a str>,
}

#[derive(Default)]
struct ExampleSourceBuilder<'a> {
    text: Option<&'a str>,
    ty: Option<&'a str>,
}

#[derive(Default)]
struct ExampleSentenceBuilder<'a> {
    text: Option<&'a str>,
    lang: Option<&'a str>,
}

#[derive(Default)]
struct SourceLanguageBuilder<'a> {
    text: Option<&'a str>,
    lang: Option<&'a str>,
    waseigo: bool,
    ty: Option<&'a str>,
}
