use std::mem;

use anyhow::{anyhow, bail, Context, Result};
use relative_path::RelativePathBuf;
use xmlparser::{ElementEnd, Token, Tokenizer};

use super::{
    Character, CodePoint, DictionaryReference, Header, Meaning, Misc, QueryCode, Radical, Reading,
    Variant,
};

#[derive(Debug)]
enum S<'a> {
    Root(Root<'a>),
    Text(&'a str, Option<&'a str>),
    AttrValue(&'a str, &'a str, AttrValue<'a>),
    Header {
        file_version: Option<&'a str>,
        database_version: Option<&'a str>,
        date_of_creation: Option<&'a str>,
    },
    Character(CharacterState<'a>),
    CodePoint(Vec<CodePoint<'a>>),
    Radical(Vec<Radical<'a>>),
    Misc(Misc<'a>),
    DictionaryReferences(Vec<DictionaryReference<'a>>),
    DictionaryReference {
        base: AttrValue<'a>,
        volume: Option<&'a str>,
        page: Option<&'a str>,
    },
    QueryCodes(Vec<QueryCode<'a>>),
    QueryCode {
        base: AttrValue<'a>,
        skip_misclass: Option<&'a str>,
    },
    ReadingMeaning {
        readings: Vec<Reading<'a>>,
        meanings: Vec<Meaning<'a>>,
        nanori: Vec<&'a str>,
    },
    ReadingMeaningGroup,
}

#[derive(Debug, Default)]
struct Root<'a> {
    header: Option<Header<'a>>,
}

#[derive(Debug, Default)]
struct CharacterState<'a> {
    literal: Option<&'a str>,
    code_points: Vec<CodePoint<'a>>,
    radicals: Vec<Radical<'a>>,
    misc: Option<Misc<'a>>,
    dictionary_references: Vec<DictionaryReference<'a>>,
    query_codes: Vec<QueryCode<'a>>,
    readings: Vec<Reading<'a>>,
    meanings: Vec<Meaning<'a>>,
    nanori: Vec<&'a str>,
}

#[derive(Debug, Default)]
struct AttrValue<'a> {
    text: Option<&'a str>,
    attr: Option<&'a str>,
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
    stack: Vec<S<'a>>,
    closed: bool,
    path: RelativePathBuf,
    tokenizer: Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            stack: Vec::new(),
            closed: false,
            path: RelativePathBuf::new(),
            tokenizer: Tokenizer::from(input),
        }
    }

    /// Parse the next entry.
    pub fn parse(&mut self) -> Result<Option<Character<'a>>> {
        self.parse_inner()
            .with_context(|| anyhow!("in /{}", self.path))
    }

    fn parse_inner(&mut self) -> Result<Option<Character<'a>>> {
        loop {
            let output = self.parse_next()?;

            match output {
                Output::Text(text) => match &mut self.stack[..] {
                    [.., S::Text(_, o @ None)] => {
                        *o = Some(text);
                    }
                    [.., S::AttrValue(_, _, AttrValue { text: o @ None, .. })] => {
                        *o = Some(text);
                    }
                    [.., S::DictionaryReference {
                        base: AttrValue { text: o @ None, .. },
                        ..
                    }] => {
                        *o = Some(text);
                    }
                    [.., S::QueryCode {
                        base: AttrValue { text: o @ None, .. },
                        ..
                    }] => {
                        *o = Some(text);
                    }
                    _ => {
                        bail!("Unexpected text {text}");
                    }
                },
                Output::Open(tag) => {
                    let state = match (tag, &self.stack[..]) {
                        ("kanjidic2", []) => S::Root(Root::default()),
                        ("header", [S::Root(..)]) => S::Header {
                            file_version: None,
                            database_version: None,
                            date_of_creation: None,
                        },
                        (
                            tag @ ("file_version" | "database_version" | "date_of_creation"),
                            [.., S::Header { .. }],
                        ) => S::Text(tag, None),
                        ("character", [S::Root(..)]) => S::Character(CharacterState::default()),
                        (tag @ "literal", [.., S::Character(..)]) => S::Text(tag, None),
                        ("codepoint", [.., S::Character(..)]) => S::CodePoint(Vec::new()),
                        ("cp_value", [.., S::CodePoint(..)]) => {
                            S::AttrValue("cp_value", "cp_type", AttrValue::default())
                        }
                        ("radical", [.., S::Character(..)]) => S::Radical(Vec::new()),
                        ("rad_value", [.., S::Radical(..)]) => {
                            S::AttrValue("rad_value", "rad_type", AttrValue::default())
                        }
                        ("misc", [.., S::Character(..)]) => S::Misc(Misc::default()),
                        (tag @ ("grade" | "stroke_count" | "freq" | "jlpt"), [.., S::Misc(..)]) => {
                            S::Text(tag, None)
                        }
                        ("variant", [.., S::Misc(..)]) => {
                            S::AttrValue("variant", "var_type", AttrValue::default())
                        }
                        ("rad_name", [.., S::Misc(..)]) => S::Text("rad_name", None),
                        ("dic_number", [.., S::Character(..)]) => {
                            S::DictionaryReferences(Vec::new())
                        }
                        ("dic_ref", [.., S::DictionaryReferences(..)]) => S::DictionaryReference {
                            base: AttrValue::default(),
                            volume: None,
                            page: None,
                        },
                        ("query_code", [.., S::Character(..)]) => S::QueryCodes(Vec::new()),
                        ("q_code", [.., S::QueryCodes(..)]) => S::QueryCode {
                            base: AttrValue::default(),
                            skip_misclass: None,
                        },
                        ("reading_meaning", [.., S::Character(..)]) => S::ReadingMeaning {
                            readings: Vec::new(),
                            meanings: Vec::new(),
                            nanori: Vec::new(),
                        },
                        ("rmgroup", [.., S::ReadingMeaning { .. }]) => S::ReadingMeaningGroup,
                        ("reading", [.., S::ReadingMeaningGroup]) => {
                            S::AttrValue("reading", "r_type", AttrValue::default())
                        }
                        ("meaning", [.., S::ReadingMeaningGroup]) => {
                            S::AttrValue("meaning", "m_lang", AttrValue::default())
                        }
                        ("nanori", [.., S::ReadingMeaning { .. }]) => S::Text("nanori", None),
                        _ => {
                            bail!("Unexpected open tag {tag}");
                        }
                    };

                    self.stack.push(state);
                }
                Output::Attribute(key, value) => match (&mut self.stack[..], key) {
                    ([.., S::AttrValue(_, ty, state)], actual) if actual == *ty => {
                        state.attr = Some(value);
                    }
                    ([.., S::DictionaryReference { base, .. }], "dr_type") => {
                        base.attr = Some(value);
                    }
                    ([.., S::DictionaryReference { volume, .. }], "m_vol") => {
                        *volume = Some(value);
                    }
                    ([.., S::DictionaryReference { page, .. }], "m_page") => {
                        *page = Some(value);
                    }
                    ([.., S::QueryCode { base, .. }], "qc_type") => {
                        base.attr = Some(value);
                    }
                    ([.., S::QueryCode { skip_misclass, .. }], "skip_misclass") => {
                        *skip_misclass = Some(value);
                    }
                    _ => {
                        bail!("Unexpected attribute {key}=\"{value}\"");
                    }
                },
                Output::Close => {
                    let head = self.stack.pop().context("Missing state")?;

                    match (&mut self.stack[..], head) {
                        (
                            [.., S::Header {
                                file_version: o @ None,
                                ..
                            }],
                            S::Text("file_version", content),
                        ) => {
                            *o = content;
                        }
                        (
                            [.., S::Header {
                                database_version: o @ None,
                                ..
                            }],
                            S::Text("database_version", content),
                        ) => {
                            *o = content;
                        }
                        (
                            [.., S::Header {
                                date_of_creation: o @ None,
                                ..
                            }],
                            S::Text("date_of_creation", content),
                        ) => {
                            *o = content;
                        }
                        (
                            [S::Root(root)],
                            S::Header {
                                file_version,
                                database_version,
                                date_of_creation,
                            },
                        ) => {
                            root.header = Some(Header {
                                file_version: file_version.context("Missing file_version")?,
                                database_version: database_version
                                    .context("Missing database_version")?,
                                date_of_creation: date_of_creation
                                    .context("Missing date_of_creation")?,
                            });
                        }
                        (
                            [.., S::Character(CharacterState {
                                literal: o @ None, ..
                            })],
                            S::Text("literal", content),
                        ) => {
                            *o = content;
                        }
                        ([.., S::CodePoint(state)], S::AttrValue("cp_value", "cp_type", value)) => {
                            state.push(CodePoint {
                                text: value.text.context("Missing cp_value")?,
                                ty: value.attr.context("Missing cp_type")?,
                            });
                        }
                        (
                            [.., S::Character(CharacterState { code_points: o, .. })],
                            S::CodePoint(value),
                        ) => {
                            *o = value;
                        }
                        ([.., S::Radical(state)], S::AttrValue("rad_value", "rad_type", value)) => {
                            state.push(Radical {
                                text: value.text.context("Missing rad_value")?,
                                ty: value.attr.context("Missing rad_type")?,
                            });
                        }
                        (
                            [.., S::Character(CharacterState { radicals: o, .. })],
                            S::Radical(value),
                        ) => {
                            *o = value;
                        }
                        (
                            [.., S::Misc(Misc {
                                grade: grade @ None,
                                ..
                            })],
                            S::Text("grade", content),
                        ) => {
                            *grade = content.map(str::parse).transpose()?;
                        }
                        (
                            [.., S::Misc(Misc { stroke_counts, .. })],
                            S::Text("stroke_count", content),
                        ) => {
                            if let Some(content) = content {
                                stroke_counts.push(content.parse()?);
                            }
                        }
                        (
                            [.., S::Misc(Misc { variant, .. })],
                            S::AttrValue("variant", "var_type", content),
                        ) => {
                            *variant = Some(Variant {
                                text: content.text.context("Missing variant")?,
                                ty: content.attr.context("Missing var_type")?,
                            });
                        }
                        (
                            [.., S::Misc(Misc {
                                freq: freq @ None, ..
                            })],
                            S::Text("freq", content),
                        ) => {
                            *freq = content.map(str::parse).transpose()?;
                        }
                        (
                            [.., S::Misc(Misc {
                                jlpt: jlpt @ None, ..
                            })],
                            S::Text("jlpt", content),
                        ) => {
                            *jlpt = content.map(str::parse).transpose()?;
                        }
                        (
                            [.., S::Misc(Misc { radical_names, .. })],
                            S::Text("rad_name", content),
                        ) => {
                            radical_names.push(content.context("Missing rad_name")?);
                        }
                        (
                            [.., S::Character(CharacterState {
                                misc: misc @ None, ..
                            })],
                            S::Misc(value),
                        ) => {
                            *misc = Some(value);
                        }
                        (
                            [.., S::DictionaryReferences(state)],
                            S::DictionaryReference { base, volume, page },
                        ) => {
                            state.push(DictionaryReference {
                                text: base.text.context("Missing dic_ref")?,
                                ty: base.attr.context("Missing dr_type")?,
                                volume,
                                page,
                            });
                        }
                        ([.., S::Character(state)], S::DictionaryReferences(references)) => {
                            state.dictionary_references = references;
                        }
                        (
                            [.., S::QueryCodes(state)],
                            S::QueryCode {
                                base,
                                skip_misclass,
                            },
                        ) => {
                            state.push(QueryCode {
                                text: base.text.context("Missing q_code")?,
                                ty: base.attr.context("Missing qc_type")?,
                                skip_misclass,
                            });
                        }
                        ([.., S::Character(state)], S::QueryCodes(query_codes)) => {
                            state.query_codes = query_codes;
                        }
                        (
                            [.., S::ReadingMeaning { readings, .. }, S::ReadingMeaningGroup],
                            S::AttrValue("reading", "r_type", attr),
                        ) => {
                            readings.push(Reading {
                                text: attr.text.context("Missing reading")?,
                                ty: attr.attr.context("Missing r_type")?,
                            });
                        }
                        (
                            [.., S::ReadingMeaning { meanings, .. }, S::ReadingMeaningGroup],
                            S::AttrValue("meaning", "m_lang", attr),
                        ) => {
                            meanings.push(Meaning {
                                text: attr.text.context("Missing reading")?,
                                lang: attr.attr,
                            });
                        }
                        ([.., S::ReadingMeaning { .. }], S::ReadingMeaningGroup) => {}
                        ([.., S::ReadingMeaning { nanori, .. }], S::Text("nanori", text)) => {
                            if let Some(text) = text {
                                nanori.push(text);
                            }
                        }
                        (
                            [.., S::Character(state)],
                            S::ReadingMeaning {
                                readings,
                                meanings,
                                nanori,
                            },
                        ) => {
                            state.readings = readings;
                            state.meanings = meanings;
                            state.nanori = nanori;
                        }
                        ([S::Root(..)], S::Character(state)) => {
                            return Ok(Some(Character {
                                literal: state.literal.context("Missing literal")?,
                                code_point: state.code_points,
                                radical: state.radicals,
                                misc: state.misc.unwrap_or_default(),
                                dictionary_references: state.dictionary_references,
                                query_codes: state.query_codes,
                                readings: state.readings,
                                meanings: state.meanings,
                                nanori: state.nanori,
                            }));
                        }
                        ([], S::Root(..)) => {
                            return Ok(None);
                        }
                        _ => {
                            bail!("Unexpected close tag at {:?}", self.stack.last());
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
