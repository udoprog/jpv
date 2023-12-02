use anyhow::{bail, Result};
use relative_path::RelativePathBuf;
use xmlparser::{ElementEnd, Token, Tokenizer};

use crate::entities::NameType;

use super::{Entry, Reading, Translation};

#[derive(Debug)]
enum State<'a> {
    Root,
    Entry(EntryBuilder<'a>),
    Kanji(Option<&'a str>),
    Reading(ReadingBuilder<'a>),
    Translations(TranslationsBuilder<'a>),
    NameType(Option<NameType>),
    Translation(TranslationBuilder<'a>),
    Text(&'static str, Option<&'a str>),
}

#[derive(Debug, Default)]
struct ReadingBuilder<'a> {
    text: Option<&'a str>,
    priority: Option<&'a str>,
}

#[derive(Debug, Default)]
struct TranslationsBuilder<'a> {
    name_types: Vec<NameType>,
    translations: Vec<Translation<'a>>,
}

#[derive(Debug, Default)]
struct TranslationBuilder<'a> {
    text: Option<&'a str>,
    lang: Option<&'a str>,
}

#[derive(Debug, Default)]
struct EntryBuilder<'a> {
    sequence: Option<u64>,
    kanji: Vec<&'a str>,
    reading: Vec<Reading<'a>>,
    name_types: Vec<NameType>,
    translations: Vec<Translation<'a>>,
}

pub struct Parser<'a> {
    stack: Vec<State<'a>>,
    path: RelativePathBuf,
    tokenizer: Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    /// Parse input.
    pub fn new(input: &'a str) -> Self {
        Self {
            stack: vec![],
            path: RelativePathBuf::new(),
            tokenizer: Tokenizer::from(input),
        }
    }

    /// Get the next parsed name.
    pub(crate) fn next(&mut self) -> Result<Option<Entry<'a>>> {
        loop {
            let Some(token) = self.tokenizer.next() else {
                if self.stack.is_empty() {
                    return Ok(None);
                }

                bail!("{}: Expected token", self.path);
            };

            let token = token?;

            macro_rules! set_option {
                ($target:expr, $value:expr) => {
                    if $target.is_some() {
                        bail!("{}: Duplicate elements", self.path);
                    }

                    *$target = $value;
                };
            }

            match token {
                Token::Declaration { .. } if self.stack.is_empty() => {}
                Token::DtdStart { .. } if self.stack.is_empty() => {}
                Token::EntityDeclaration { .. } if self.stack.is_empty() => {}
                Token::DtdEnd { .. } if self.stack.is_empty() => {}
                Token::Comment { .. } => {}
                Token::ElementStart { local, .. } => {
                    match (&self.stack[..], local.as_str()) {
                        ([], "JMnedict") => {
                            self.stack.push(State::Root);
                        }
                        ([State::Root], "entry") => {
                            self.stack.push(State::Entry(EntryBuilder::default()));
                        }
                        ([.., State::Entry(..)], "ent_seq") => {
                            self.stack.push(State::Text("ent_seq", None));
                        }
                        ([.., State::Entry(..)], "k_ele") => {
                            self.stack.push(State::Kanji(None));
                        }
                        ([.., State::Kanji(..)], "keb") => {
                            self.stack.push(State::Text("keb", None));
                        }
                        ([.., State::Entry(..)], "r_ele") => {
                            self.stack.push(State::Reading(ReadingBuilder::default()));
                        }
                        ([.., State::Reading(..)], "reb") => {
                            self.stack.push(State::Text("reb", None));
                        }
                        ([.., State::Reading(..)], "re_pri") => {
                            self.stack.push(State::Text("re_pri", None));
                        }
                        ([.., State::Entry(..)], "trans") => {
                            self.stack
                                .push(State::Translations(TranslationsBuilder::default()));
                        }
                        ([.., State::Translations(..)], "name_type") => {
                            self.stack.push(State::NameType(None));
                        }
                        ([.., State::Translations(..)], "trans_det") => {
                            self.stack
                                .push(State::Translation(TranslationBuilder::default()));
                        }
                        (_, element) => {
                            bail!("{}: Unsupported open element: {}", self.path, element);
                        }
                    }

                    self.path.push(local.as_str());
                }
                Token::ElementEnd { end, .. } => {
                    let actual = match end {
                        ElementEnd::Open => continue,
                        ElementEnd::Close(_, name) => Some(name.as_str()),
                        ElementEnd::Empty => None,
                    };

                    macro_rules! expect_close {
                        ($name:expr) => {
                            if let Some(actual) = actual {
                                if actual != $name {
                                    bail!(
                                        "{}: Expected close element `{}` but got `{actual}`",
                                        self.path,
                                        $name
                                    );
                                }
                            }
                        };
                    }

                    let Some(top) = self.stack.pop() else {
                        bail!("{}: Missing state", self.path);
                    };

                    match (&mut self.stack[..], top) {
                        ([State::Root], State::Entry(entry)) => {
                            let Some(sequence) = entry.sequence else {
                                bail!("{}: Missing sequence", self.path);
                            };

                            self.path.pop();

                            return Ok(Some(Entry {
                                sequence,
                                kanji: entry.kanji,
                                reading: entry.reading,
                                name_types: entry.name_types,
                                translations: entry.translations,
                            }));
                        }
                        ([.., State::Entry(entry)], State::Text("ent_seq", value)) => {
                            expect_close!("ent_seq");

                            let Ok(value) = value.map(str::parse).transpose() else {
                                bail!("{}: Invalid sequence", self.path);
                            };

                            set_option!(&mut entry.sequence, value);
                        }
                        ([.., State::Entry(entry)], State::Kanji(value)) => {
                            expect_close!("k_ele");

                            let Some(value) = value else {
                                bail!("{}: Missing text", self.path)
                            };

                            entry.kanji.push(value);
                        }
                        ([.., State::Kanji(entry)], State::Text("keb", value)) => {
                            expect_close!("keb");
                            set_option!(entry, value);
                        }
                        ([.., State::Entry(entry)], State::Reading(value)) => {
                            expect_close!("r_ele");

                            let Some(text) = value.text else {
                                bail!("{}: Missing text", self.path)
                            };

                            entry.reading.push(Reading {
                                text,
                                priority: value.priority,
                            });
                        }
                        ([.., State::Entry(entry)], State::Translations(value)) => {
                            expect_close!("trans");
                            entry.name_types.extend(value.name_types);
                            entry.translations.extend(value.translations);
                        }
                        ([.., State::Reading(entry)], State::Text("reb", value)) => {
                            expect_close!("reb");
                            set_option!(&mut entry.text, value);
                        }
                        ([.., State::Reading(entry)], State::Text("re_pri", value)) => {
                            expect_close!("re_pri");
                            set_option!(&mut entry.priority, value);
                        }
                        ([.., State::Translations(entry)], State::NameType(value)) => {
                            expect_close!("name_type");

                            let Some(value) = value else {
                                bail!("{}: Missing name type", self.path);
                            };

                            entry.name_types.push(value);
                        }
                        ([.., State::Translations(entry)], State::Translation(value)) => {
                            expect_close!("trans_det");

                            let Some(text) = value.text else {
                                bail!("{}: Missing text", self.path)
                            };

                            entry.translations.push(Translation {
                                text,
                                lang: value.lang,
                            });
                        }
                        ([], State::Root) => {
                            expect_close!("JMnedict");
                        }
                        (_, top) => {
                            bail!("{}: Unexpected close element: {top:?}", self.path);
                        }
                    }

                    self.path.pop();
                }
                Token::Text { text } => {
                    let text = text.as_str().trim();

                    if text.is_empty() {
                        continue;
                    }

                    match &mut self.stack[..] {
                        [.., State::Text(_, value)] => {
                            *value = Some(text);
                        }
                        [.., State::NameType(value)] => {
                            let Some(name_type) = NameType::parse(text) else {
                                bail!("{}: Unsupported name type: {text}", self.path);
                            };

                            set_option!(value, Some(name_type));
                        }
                        [.., State::Translation(translation)] => {
                            set_option!(&mut translation.text, Some(text));
                        }
                        _ => {
                            bail!("{}: Unexpected text: {text}", self.path);
                        }
                    }
                }
                token => {
                    bail!("{}: Unsupported token: {:?}", self.path, token);
                }
            }
        }
    }
}
