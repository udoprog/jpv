use core::fmt;
use core::mem;

use anyhow::{anyhow, ensure, Context, Result};
use fixed_map::Set;

use crate::elements::{gloss, source_language, text};
use crate::elements::{Gloss, SourceLanguage};
use crate::entities::Dialect;
use crate::entities::Field;
use crate::entities::{Miscellaneous, PartOfSpeech};
use crate::parser::{Output, Poll};

const DEFAULT_LANGUAGE: &str = "eng";

#[derive(Debug)]
pub struct Sense<'a> {
    pub part_of_speech: Set<PartOfSpeech>,
    pub xref: Option<&'a str>,
    pub gloss: Vec<Gloss<'a>>,
    pub info: Option<&'a str>,
    pub misc: Set<Miscellaneous>,
    pub dialects: Set<Dialect>,
    pub stagk: Vec<&'a str>,
    pub stagr: Vec<&'a str>,
    pub fields: Set<Field>,
    pub source_language: Vec<SourceLanguage<'a>>,
    pub antonym: Vec<&'a str>,
}

impl<'a> Sense<'a> {
    /// Debug the sense element, while avoiding formatting elements which are
    /// not defined.
    pub fn debug_sparse(&self) -> impl fmt::Debug + '_ {
        DebugSparse(self)
    }

    pub fn is_lang(&self, arg: &str) -> bool {
        for g in &self.gloss {
            if g.lang.unwrap_or(DEFAULT_LANGUAGE) == arg {
                return true;
            }
        }

        false
    }
}

struct DebugSparse<'a>(&'a Sense<'a>);

impl fmt::Debug for DebugSparse<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct SparseDebug<'a>(&'a [SourceLanguage<'a>]);

        impl<'a> fmt::Debug for SparseDebug<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut f = f.debug_list();

                for item in self.0 {
                    f.entry(&item.debug_sparse());
                }

                f.finish()
            }
        }

        let mut f = f.debug_struct("Sense");

        if !self.0.part_of_speech.is_empty() {
            f.field("part_of_speech", &self.0.part_of_speech);
        }

        if let Some(field) = self.0.xref {
            f.field("xref", &field);
        }

        if let Some(field) = self.0.info {
            f.field("info", &field);
        }

        if !self.0.misc.is_empty() {
            f.field("misc", &self.0.misc);
        }

        if !self.0.dialects.is_empty() {
            f.field("dialect", &self.0.dialects);
        }

        if !self.0.stagk.is_empty() {
            f.field("stagk", &self.0.stagk);
        }

        if !self.0.stagr.is_empty() {
            f.field("stagr", &self.0.stagr);
        }

        if !self.0.fields.is_empty() {
            f.field("fields", &self.0.fields);
        }

        if !self.0.source_language.is_empty() {
            f.field("source_language", &SparseDebug(&self.0.source_language));
        }

        if !self.0.antonym.is_empty() {
            f.field("antonym", &self.0.antonym);
        }

        f.finish_non_exhaustive()
    }
}

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    Pos(text::Builder<'a>),
    Xref(text::Builder<'a>),
    Gloss(gloss::Builder<'a>),
    Information(text::Builder<'a>),
    Misc(text::Builder<'a>),
    Dialect(text::Builder<'a>),
    StagK(text::Builder<'a>),
    StagR(text::Builder<'a>),
    Field(text::Builder<'a>),
    SourceLanguage(source_language::Builder<'a>),
    Antonym(text::Builder<'a>),
}

#[derive(Debug, Default)]
pub(super) struct Builder<'a> {
    state: State<'a>,
    part_of_speech: Set<PartOfSpeech>,
    xref: Option<&'a str>,
    gloss: Vec<Gloss<'a>>,
    info: Option<&'a str>,
    misc: Set<Miscellaneous>,
    dialects: Set<Dialect>,
    stagk: Vec<&'a str>,
    stagr: Vec<&'a str>,
    fields: Set<Field>,
    source_language: Vec<SourceLanguage<'a>>,
    antonym: Vec<&'a str>,
}

impl<'a> Builder<'a> {
    builder! {
        self => Sense<'a> {
            "pos", Pos, value => {
                self.part_of_speech.insert(PartOfSpeech::parse(value).with_context(|| anyhow!("Unsupported part of speech `{}`", value))?);
            },
            "xref", Xref, value => {
                self.xref = Some(value);
            },
            "gloss", Gloss, value => {
                self.gloss.push(value);
            },
            "s_inf", Information, value => {
                ensure!(self.info.is_none(), "info already set");
                self.info = Some(value);
            },
            "misc", Misc, value => {
                let misc = Miscellaneous::parse(value).with_context(|| anyhow!("Unsupported misc `{value}`"))?;
                self.misc.insert(misc);
            },
            "dial", Dialect, value => {
                let dialect = Dialect::parse(value).with_context(|| anyhow!("Unsupported dialect `{value}`"))?;
                self.dialects.insert(dialect);
            },
            "stagk", StagK, value => {
                self.stagk.push(value);
            },
            "stagr", StagR, value => {
                self.stagr.push(value);
            },
            "field", Field, value => {
                let field = Field::parse(value).with_context(|| anyhow!("Unsupported field `{value}`"))?;
                self.fields.insert(field);
            },
            "lsource", SourceLanguage, value => {
                self.source_language.push(value);
            },
            "ant", Antonym, value => {
                self.antonym.push(value);
            },
        }
    }

    fn build(&mut self) -> Result<Sense<'a>> {
        let gloss = mem::take(&mut self.gloss);
        let part_of_speech = mem::take(&mut self.part_of_speech);
        let misc = mem::take(&mut self.misc);
        let dialects = mem::take(&mut self.dialects);
        let stagk = mem::take(&mut self.stagk);
        let stagr = mem::take(&mut self.stagr);
        let fields = mem::take(&mut self.fields);
        let source_language = mem::take(&mut self.source_language);
        let antonym = mem::take(&mut self.antonym);

        Ok(Sense {
            part_of_speech,
            xref: self.xref,
            gloss,
            info: self.info,
            misc,
            dialects,
            stagk,
            stagr,
            fields,
            source_language,
            antonym,
        })
    }
}
