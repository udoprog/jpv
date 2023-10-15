use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};
use std::fmt;
use std::io::Write;
use std::mem;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use lib::adjective;
use lib::database::{Database, IndexSource};
use lib::verb;
use lib::{Form, Furigana, PartOfSpeech};
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
    /// Perform inflection.
    #[arg(long)]
    inflection: bool,
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
    /// Include polite variants of inflections.
    #[arg(long)]
    polite: bool,
    /// Only fetch the specified sequence ids.
    #[arg(long = "seq")]
    sequences: Vec<u32>,
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

fn main() -> Result<()> {
    let filter = EnvFilter::builder().from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let database_path = Path::new("database.bin");

    let args = Args::try_parse()?;

    if args.list_pos {
        println!("Available `--pos` arguments:");

        for pos in PartOfSpeech::VALUES {
            println!("{} - {} / {}", pos.ident(), pos.variant(), pos.help());
        }

        return Ok(());
    }

    let data =
        load_database(database_path).with_context(|| anyhow!("{}", database_path.display()))?;

    let db = Database::new(data.as_ref())?;

    let mut to_look_up = BTreeSet::new();

    for &seq in &args.sequences {
        to_look_up.extend(db.lookup_sequence(seq)?);
    }

    for input in &args.arguments {
        let seed = args.sequences.is_empty();

        if seed {
            to_look_up.extend(db.lookup(input)?);
        } else {
            let filter = db
                .lookup(input)?
                .into_iter()
                .map(|id| id.index())
                .collect::<HashSet<_>>();
            to_look_up.retain(|id| filter.contains(&id.index()));
        }
    }

    if !args.parts_of_speech.is_empty() {
        let mut seed = args.arguments.is_empty() && args.sequences.is_empty();

        for pos in &args.parts_of_speech {
            let pos = PartOfSpeech::parse_keyword(pos)
                .with_context(|| anyhow!("Invalid part of speech `{pos}`"))?;

            let indexes = db.by_pos(pos)?;

            if mem::take(&mut seed) {
                to_look_up.extend(indexes);
                continue;
            }

            to_look_up.retain(|index| indexes.contains(index));
        }
    }

    let current_lang = args.lang.as_deref().unwrap_or("eng");

    for (i, index) in to_look_up.iter().enumerate() {
        let extra = match index.source() {
            IndexSource::VerbInflection { inflection, .. } => {
                Some(format!("Found through verb inflection: {inflection:?}"))
            }
            IndexSource::AdjectiveInflection { inflection } => Some(format!(
                "Found through adjective inflection: {inflection:?}"
            )),
            _ => None,
        };

        let d = db.get(*index)?;

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

            for g in &sense.gloss {
                if let Some(lang) = g.lang {
                    println!("  - {} ({lang})", g.text);
                } else {
                    println!("  - {}", g.text);
                }
            }

            if args.examples && !sense.examples.is_empty() {
                println!("  Examples:");

                for e in &sense.examples {
                    println!("  - {e:?}");
                }
            }
        }

        if !args.inflection || (to_look_up.len() > 1 && args.sequences.is_empty()) {
            continue;
        }

        let p = "  ";

        let dis0 = |furigana| maybe_furigana(furigana, !args.no_furigana);
        let dis = |furigana| maybe_furigana(furigana, !args.no_furigana);

        let stdout = std::io::stdout();
        let mut o = stdout.lock();

        for (_, c) in verb::conjugate(&d) {
            writeln!(o, "{p}# Inflections:")?;

            writeln!(o, "{p}  Dictionary:")?;
            writeln!(o, "{p}  - {}", dis0(c.dictionary.furigana()))?;

            for (c, form) in c.inflections {
                if args.polite != c.contains(Form::Polite) {
                    continue;
                }

                writeln!(o, "{p}  {c:?}:")?;
                writeln!(o, "{p}  - {}", dis(form.furigana()))?;
            }
        }

        if let Some(c) = adjective::conjugate(&d) {
            writeln!(o, "{p}# Inflections:")?;

            writeln!(o, "{p}  Dictionary:")?;
            writeln!(o, "{p}  - {}", dis0(c.dictionary.furigana()))?;

            for (c, form) in c.inflections {
                if args.polite != c.contains(Form::Polite) {
                    continue;
                }

                writeln!(o, "{p}  {c:?}:")?;
                writeln!(o, "{p}  - {}", dis(form.furigana()))?;
            }
        }

        o.flush()?;
    }

    Ok(())
}

fn maybe_furigana<const N: usize, const S: usize>(
    furigana: Furigana<'_, N, S>,
    do_furigana: bool,
) -> impl fmt::Display + '_ {
    struct Display<'a, const N: usize, const S: usize> {
        furigana: Furigana<'a, N, S>,
        do_furigana: bool,
    }

    impl<const N: usize, const S: usize> fmt::Display for Display<'_, N, S> {
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
