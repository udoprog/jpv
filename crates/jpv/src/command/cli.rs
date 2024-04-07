#![allow(clippy::too_many_arguments)]

use std::collections::{BTreeSet, HashSet};
use std::fmt;
use std::fs;
use std::io::Write;
use std::mem;
use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;
use lib::config::Config;
use lib::data;
use lib::database::{Database, Entry, Id};
use lib::inflection;
use lib::{Dirs, Form, Furigana, PartOfSpeech};

use crate::Args;

#[derive(Parser)]
pub(crate) struct CliArgs {
    #[arg(long)]
    long: Option<PathBuf>,
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
    /// Include polite variants of inflections.
    #[arg(long)]
    polite: bool,
    /// Only fetch the specified sequence ids.
    #[arg(long = "seq")]
    sequences: Vec<u32>,
    /// Output format to use, defaults to `json`. Available options are: rich,
    /// json, json-pretty.
    #[arg(long)]
    output_format: Option<String>,
    /// Search arguments to filter by. Must be either kana or kanji, which is
    /// matched against entries searched for.
    #[arg(name = "arguments")]
    arguments: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Rich,
    Json,
    JsonPretty,
}

pub(crate) async fn run(
    args: &Args,
    cli_args: &CliArgs,
    dirs: &Dirs,
    config: Config,
) -> Result<()> {
    let format = match cli_args.output_format.as_deref() {
        Some("rich") => OutputFormat::Rich,
        Some("json") | None => OutputFormat::Json,
        Some("json-pretty") => OutputFormat::JsonPretty,
        Some(name) => bail!("Unsupported output format: {}", name),
    };

    if cli_args.list_pos {
        println!("Available `--pos` arguments:");

        for pos in PartOfSpeech::VALUES {
            println!("{} - {} / {}", pos.ident(), pos.variant(), pos.help());
        }

        return Ok(());
    }

    // SAFETY: we know this is only initialized once here exclusively.
    let indexes = data::open_from_args(&args.index[..], dirs)?;
    let db = Database::open(indexes, &config)?;

    if let Some(path) = &cli_args.long {
        let ids = db.all()?;

        let mut f = fs::File::create(path)?;

        let mut uniq = HashSet::new();

        for id in ids {
            let Entry::Phrase(entry) = db.entry_at(id)? else {
                continue;
            };

            for (kanji, (_, reading), _) in lib::inflection::reading_permutations(&entry) {
                let Some((_, kanji)) = kanji else {
                    continue;
                };

                for group in lib::Furigana::new(kanji, reading, "").iter() {
                    let lib::FuriganaGroup::Kanji(kanji, reading) = group else {
                        continue;
                    };

                    if kanji.chars().count() != 1 || lib::morae::iter(reading).count() < 4 {
                        continue;
                    }

                    if uniq.insert((kanji, reading)) {
                        writeln!(f, "#{}: {kanji}: {reading}", entry.sequence)?;
                    }

                    if let Some(e) = db.literal_to_kanji(kanji)? {
                        for reading in e.readings {
                            if matches!(reading.ty, "ja_kun" | "ja_on") {
                                writeln!(f, "  {}: {}", reading.ty, reading.text)?;
                            }
                        }
                    };
                }
            }
        }

        return Ok(());
    }

    let mut to_look_up = BTreeSet::new();

    for &seq in &cli_args.sequences {
        to_look_up.extend(db.sequence_to_id(seq)?);
    }

    for input in &cli_args.arguments {
        let seed = cli_args.sequences.is_empty();

        if seed {
            to_look_up.extend(db.lookup(input)?);
        } else {
            let filter = db
                .lookup(input)?
                .into_iter()
                .map(|id| id.key())
                .collect::<HashSet<_>>();
            to_look_up.retain(|id| filter.contains(&id.key()));
        }
    }

    if !cli_args.parts_of_speech.is_empty() {
        let mut seed = cli_args.arguments.is_empty() && cli_args.sequences.is_empty();
        let mut pos = fixed_map::Set::new();

        for p in cli_args
            .parts_of_speech
            .iter()
            .flat_map(|s| PartOfSpeech::parse_keyword(s))
        {
            pos.insert(p);
        }

        let ids = db.by_pos(pos)?;

        if mem::take(&mut seed) {
            to_look_up.extend(ids);
        } else {
            to_look_up.retain(|id| ids.contains(id));
        }
    }

    let current_lang = cli_args.lang.as_deref().unwrap_or("eng");

    let o = std::io::stdout();
    let mut o = o.lock();

    for (i, id) in to_look_up.iter().enumerate() {
        match format {
            OutputFormat::Rich => {
                print_rich(&mut o, &db, cli_args, current_lang, &to_look_up, i, *id)?
            }
            OutputFormat::Json | OutputFormat::JsonPretty => print_json(
                &mut o,
                &db,
                cli_args,
                matches!(format, OutputFormat::JsonPretty),
                i,
                id,
            )?,
        }
    }

    Ok(())
}

fn print_rich<O>(
    o: &mut O,
    db: &Database,
    cli_args: &CliArgs,
    current_lang: &str,
    to_look_up: &BTreeSet<Id>,
    i: usize,
    id: Id,
) -> Result<()>
where
    O: ?Sized + Write,
{
    match db.entry_at(id)? {
        Entry::Phrase(d) => {
            println!("#{i} Sequence: {}", d.sequence);

            for (index, reading) in d.reading_elements.iter().enumerate() {
                println!("  #{index} {:?}", reading.debug_sparse());
            }

            for (index, kanji) in d.kanji_elements.iter().enumerate() {
                println!("  #{index} {:?}", kanji.debug_sparse());
            }

            for (index, sense) in d.senses.iter().enumerate() {
                if !cli_args.any_lang && !sense.is_lang(current_lang) {
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

                if cli_args.examples && !sense.examples.is_empty() {
                    println!("  Examples:");

                    for e in &sense.examples {
                        println!("  - {e:?}");
                    }
                }
            }

            if !cli_args.inflection || (to_look_up.len() > 1 && cli_args.sequences.is_empty()) {
                return Ok(());
            }

            let p = "  ";

            for (_, c, _) in inflection::conjugate(&d) {
                writeln!(o, "{p}# Inflections:")?;

                writeln!(o, "{p}  Dictionary:")?;
                writeln!(
                    o,
                    "{p}  - {}",
                    maybe_furigana(c.dictionary.furigana(), !cli_args.no_furigana)
                )?;

                for (c, form) in c.inflections {
                    if cli_args.polite != c.contains(Form::Honorific) {
                        continue;
                    }

                    let fur = form.furigana();
                    writeln!(o, "{p}  {c:?}:")?;
                    writeln!(
                        o,
                        "{p}  - {}",
                        maybe_furigana(fur.borrow(), !cli_args.no_furigana)
                    )?;
                }
            }
        }
        Entry::Kanji(kanji) => {
            writeln!(o, "Kanji: {}", kanji.literal)?;

            for reading in kanji.readings {
                writeln!(o, "{}: {}", reading.ty, reading.text)?;
            }

            for meaning in kanji.meanings {
                if let Some(lang) = meaning.lang {
                    writeln!(o, "{lang}: {}", meaning.text)?;
                } else {
                    writeln!(o, "{}", meaning.text)?;
                }
            }
        }
        Entry::Name(entry) => {
            for kanji in entry.kanji.iter() {
                writeln!(o, "Kanji: {}", kanji)?;
            }

            for reading in entry.reading.iter() {
                writeln!(o, "Reading: {}", reading.text)?;
            }
        }
        _ => {
            writeln!(o, "Unsupported entry")?;
        }
    }

    o.flush()?;
    Ok(())
}

fn print_json<O>(
    o: &mut O,
    db: &Database,
    _: &CliArgs,
    pretty: bool,
    _: usize,
    id: &Id,
) -> Result<()>
where
    O: ?Sized + Write,
{
    let output = db.entry_at(*id)?;

    if pretty {
        serde_json::to_writer_pretty(&mut *o, &output)?;
    } else {
        serde_json::to_writer(&mut *o, &output)?;
    }

    writeln!(o)?;
    Ok(())
}

fn maybe_furigana(furigana: Furigana<'_>, do_furigana: bool) -> impl fmt::Display + '_ {
    struct Display<'a> {
        furigana: Furigana<'a>,
        do_furigana: bool,
    }

    impl fmt::Display for Display<'_> {
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
