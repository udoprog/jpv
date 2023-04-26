use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt;
use std::mem;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use lib::database::{Database, IndexExtra, IndexRef};
use lib::verb;
use lib::{Furigana, PartOfSpeech};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
struct Args {
    /// Filter by parts of speech. If no arguments are specified, will filter by
    /// entries which matches all specified parts of speech.
    #[arg(long = "pos", name = "pos")]
    parts_of_speech: Vec<String>,
    /// List available parts of speech options an exit.
    #[arg(long)]
    list_pos: bool,
    /// Show conjugations for results.
    #[arg(long)]
    conjugate: bool,
    /// Show examples for results.
    #[arg(long)]
    examples: bool,
    /// Show glossary entries for the specified language. Defaults to "eng".
    #[arg(long)]
    lang: Option<String>,
    /// Show glossary entries for any language. Overrides `--lang <lang>`.
    #[arg(long)]
    any_lang: bool,
    /// Don't print output in furigana.
    #[arg(long)]
    no_furigana: bool,
    /// Search arguments to filter by. Must be either kana or kanji, which is
    /// matched against entries searched for.
    #[arg(name = "arguments")]
    arguments: Vec<String>,
}

#[cfg(not(feature = "embed"))]
#[inline]
fn load_database(path: &Path) -> Result<Cow<'static, [u8]>> {
    Ok(Cow::Owned(std::fs::read(path)?))
}

#[cfg(feature = "embed")]
#[inline]
fn load_database(_: &Path) -> Result<Cow<'static, [u8]>> {
    const BYTES: &[u8] = include_bytes!("../../../database.bin");
    Ok(Cow::Borrowed(BYTES))
}

#[cfg(not(feature = "embed"))]
#[inline]
fn load_index(path: &Path) -> Result<Cow<'static, [u8]>> {
    let index = std::fs::read(path)?;
    Ok(Cow::Owned(index))
}

#[cfg(feature = "embed")]
#[inline]
fn load_index(_: &Path) -> Result<Cow<'static, [u8]>> {
    const BYTES: &[u8] = include_bytes!("../../../index.bin");
    Ok(Cow::Borrowed(BYTES))
}

fn main() -> Result<()> {
    let filter = EnvFilter::builder().from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let database_path = Path::new("database.bin");
    let index_path = Path::new("index.bin");

    let args = Args::try_parse()?;

    if args.list_pos {
        println!("Available `--pos` arguments:");

        for pos in PartOfSpeech::VALUES {
            println!("{} - {} / {}", pos.ident(), pos.variant(), pos.help());
        }

        return Ok(());
    }

    let data =
        load_database(&database_path).with_context(|| anyhow!("{}", database_path.display()))?;

    let index = load_index(&index_path).with_context(|| anyhow!("{}", index_path.display()))?;
    let index = IndexRef::from_bytes(index.as_ref())?;

    let db = Database::new(data.as_ref(), index);

    let mut to_look_up = BTreeSet::new();

    for input in &args.arguments {
        for index in db.lookup(&input) {
            to_look_up.insert(index);
        }
    }

    if !args.parts_of_speech.is_empty() {
        let mut seed = args.arguments.is_empty();

        for pos in &args.parts_of_speech {
            let pos = PartOfSpeech::parse_keyword(pos)
                .with_context(|| anyhow!("Invalid part of speech `{pos}`"))?;

            let indexes = db.by_pos(pos);

            if mem::take(&mut seed) {
                to_look_up.extend(indexes);
                continue;
            }

            to_look_up.retain(|index| indexes.contains(index));
        }
    }

    let current_lang = args.lang.as_deref().unwrap_or("eng");

    for (i, index) in to_look_up.into_iter().enumerate() {
        let extra = match index.extra() {
            IndexExtra::Conjugation(conjugation) => {
                Some(format!("Found through conjugation: {conjugation:?}"))
            }
            _ => None,
        };

        let d = db.get(index)?;

        if let Some(extra) = extra {
            println!("{extra}");
        }

        println!("#{i} Sequence: {}", d.sequence);

        for (index, reading) in d.reading_elements.iter().enumerate() {
            println!("  #{index} {:?}", reading.debug_sparse());
        }

        for (index, kanji) in d.kanji_elements.iter().enumerate() {
            println!("  #{index} {:?}", kanji.debug_sparse());
        }

        for (index, sense) in d.senses.iter().enumerate() {
            if !args.any_lang && !sense.is_lang(current_lang) {
                continue;
            }

            println!("  #{index} {:?}", sense.debug_sparse());

            for (i, g) in sense.gloss.iter().enumerate() {
                if let Some(lang) = g.lang {
                    println!("    #{i}: {} ({lang})", g.text);
                } else {
                    println!("    #{i}: {}", g.text);
                }
            }

            if args.examples {
                for e in &sense.examples {
                    println!("    {e:?}");
                }
            }
        }

        if !args.conjugate {
            continue;
        }

        let dis0 = |furigana| maybe_furigana::<1>(furigana, !args.no_furigana);
        let dis = |furigana| maybe_furigana::<2>(furigana, !args.no_furigana);

        if let Some(c) = verb::conjugate(&d) {
            println!("# Conjugations:");

            println!("  Dictionary 終止形 (しゅうしけい) / Present / Future / Attributive:");
            println!("    {}", dis0(c.dictionary.furigana()));

            if let Some(form) = c.get(verb::Conjugation::Te) {
                println!("  ~ Te:");
                println!("    {}", dis(form.furigana()));
            }

            if let Some(form) = c.get(verb::Conjugation::Negative) {
                println!("  Negative Short 未線形 (みぜんけい):");
                println!("    {}", dis(form.furigana()));
            }

            if c.has_polite() {
                println!("  Polite 連用形 (れんようけい):");

                if let Some(form) = c.get(verb::Conjugation::PoliteIndicative) {
                    println!("  ~ Present:");
                    println!("    {}", dis(form.furigana()));
                }

                if let Some(form) = c.get(verb::Conjugation::PoliteNegative) {
                    println!("  ~ Present Negative:");
                    println!("    {}", dis(form.furigana()));
                }

                if let Some(form) = c.get(verb::Conjugation::PolitePast) {
                    println!("  ~ Past:");
                    println!("    {}", dis(form.furigana()));
                }

                if let Some(form) = c.get(verb::Conjugation::PolitePastNegative) {
                    println!("  ~ Past Negative:");
                    println!("    {}", dis(form.furigana()));
                }
            }

            if let Some(form) = c.get(verb::Conjugation::Past) {
                println!("  Past:");
                println!("    {}", dis(form.furigana()));
            }

            if let Some(form) = c.get(verb::Conjugation::PastNegative) {
                println!("  Past Negative:");
                println!("    {}", dis(form.furigana()));
            }

            if let Some(form) = c.get(verb::Conjugation::Hypothetical) {
                println!("  Hypothetical / Conditional 仮定形 (かていけい):");
                println!("    {}", dis(form.furigana()));
            }

            if let Some(form) = c.get(verb::Conjugation::Conditional) {
                println!("  Conditional:");
                println!("    {}", dis(form.furigana()));
            }

            if let Some(form) = c.get(verb::Conjugation::Potential) {
                println!("  Potential 可能形 (かのうけい):");
                println!("    {}", dis(form.furigana()));

                if let Some(form) = c.get(verb::Conjugation::PotentialAlt) {
                    println!("    ~ {} (conversational)", dis(form.furigana()));
                }
            }

            if let Some(form) = c.get(verb::Conjugation::Command) {
                println!("  Command/Imperative 命令形 (めいれいけい):");
                println!("    {}", dis(form.furigana()));

                if let Some(form) = c.get(verb::Conjugation::CommandAlt) {
                    println!("    ~ {} (alternate)", dis(form.furigana()));
                }
            }

            if let Some(form) = c.get(verb::Conjugation::Volitional) {
                println!("  Volitional 意向形 (いこうけい):");
                println!("    {}", dis(form.furigana()));
            }

            if let Some(form) = c.get(verb::Conjugation::Passive) {
                println!("  Passive:");
                println!("    {}", dis(form.furigana()));
            }

            if let Some(form) = c.get(verb::Conjugation::Causative) {
                println!("  Causative:");
                println!("    {}", dis(form.furigana()));
            }

            if let Some(form) = c.get(verb::Conjugation::Tai) {
                println!("  Tai:");
                println!("    {}", dis(form.furigana()));
                println!("    note: can be further conjugated as i-adjective");
            }
        }
    }

    Ok(())
}

fn maybe_furigana<const N: usize>(
    furigana: Furigana<'_, N>,
    do_furigana: bool,
) -> impl fmt::Display + '_ {
    struct Display<'a, const N: usize> {
        furigana: Furigana<'a, N>,
        do_furigana: bool,
    }

    impl<const N: usize> fmt::Display for Display<'_, N> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.do_furigana {
                self.furigana.fmt(f)
            } else if self.furigana.kanji() != self.furigana.reading() {
                write!(f, "{} ({})", self.furigana.kanji(), self.furigana.reading())
            } else {
                write!(f, "{}", self.furigana.kanji())
            }
        }
    }

    Display {
        furigana,
        do_furigana,
    }
}
