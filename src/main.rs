#![allow(clippy::large_enum_variant)]

mod composite;
mod elements;
mod entities;
mod parser;
mod priority;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::Read;
use std::mem;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use elements::entry::{Conjugation, Polite};
use flate2::read::GzDecoder;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::composite::comp;
use crate::entities::PartOfSpeech;

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
    /// Search arguments to filter by. Must be either kana or kanji, which is
    /// matched against entries searched for.
    #[arg(name = "arguments")]
    arguments: Vec<String>,
}

#[cfg(debug_assertions)]
fn load_dict() -> Result<String> {
    use std::fs::File;

    let input = File::open("JMdict_e_examp.gz").context("JMdict_e_examp.gz")?;
    let mut input = GzDecoder::new(input);
    let mut string = String::new();
    input.read_to_string(&mut string)?;
    Ok(string)
}

#[cfg(not(debug_assertions))]
fn load_dict() -> Result<String> {
    static DICT: &[u8] = include_bytes!("../JMdict_e_examp.gz");
    let mut input = GzDecoder::new(std::io::Cursor::new(DICT));
    let mut string = String::new();
    input.read_to_string(&mut string)?;
    Ok(string)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Index {
    /// An exact dictionary index.
    Exact(usize),
    /// A lookup based on a conjugation.
    VerbConjugation(usize, Polite, Conjugation),
}

fn main() -> Result<()> {
    let filter = EnvFilter::builder().from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let args = Args::try_parse()?;

    if args.list_pos {
        println!("Available `--pos` arguments:");

        for pos in PartOfSpeech::VALUES {
            println!("{} - {} / {}", pos.ident(), pos.variant(), pos.help());
        }

        return Ok(());
    }

    let dict = load_dict()?;

    let start = Instant::now();

    let mut database = Vec::new();
    let mut lookup = HashMap::<_, Vec<Index>>::new();
    let mut by_pos = HashMap::<_, HashSet<usize>>::new();

    let mut parser = parser::Parser::new(&dict);

    while let Some(entry) = parser.parse()? {
        tracing::trace!(?entry);

        let index = database.len();

        for sense in &entry.senses {
            for pos in &sense.pos {
                by_pos.entry(pos).or_default().insert(index);
            }

            for g in &sense.gloss {
                for part in g.text.split_whitespace() {
                    let part = part.trim();

                    lookup
                        .entry(comp([part]))
                        .or_default()
                        .push(Index::Exact(index));
                }
            }
        }

        for el in &entry.reading_elements {
            lookup
                .entry(comp([el.text]))
                .or_default()
                .push(Index::Exact(index));
        }

        for el in &entry.kanji_elements {
            lookup
                .entry(comp([el.text]))
                .or_default()
                .push(Index::Exact(index));
        }

        if let Some(c) = entry.as_verb_conjugation() {
            for (polite, kind, phrase) in c.iter() {
                lookup
                    .entry(phrase)
                    .or_default()
                    .push(Index::VerbConjugation(index, polite, kind));
            }
        }

        database.push(entry);
    }

    let duration = Instant::now().duration_since(start);
    tracing::info!(?duration);

    let mut to_look_up = BTreeSet::new();

    for input in &args.arguments {
        let Some(indexes) = lookup.get(&comp([input.as_str()])) else {
            println!("nothing for `{input}`");
            continue;
        };

        to_look_up.extend(indexes.iter().copied());
    }

    if !args.parts_of_speech.is_empty() {
        let mut seed = args.arguments.is_empty();

        for pos in &args.parts_of_speech {
            let pos = PartOfSpeech::parse_keyword(pos)
                .with_context(|| anyhow!("Invalid part of speech `{pos}`"))?;

            let Some(indexes) = by_pos.get(&pos) else {
                continue;
            };

            if mem::take(&mut seed) {
                to_look_up.extend(indexes.iter().copied().map(Index::Exact));
                continue;
            }

            to_look_up.retain(|index| {
                let index = match index {
                    Index::Exact(index) => index,
                    Index::VerbConjugation(index, _, _) => index,
                };

                indexes.contains(index)
            });
        }
    }

    let current_lang = args.lang.as_deref().unwrap_or("eng");

    for (i, index) in to_look_up.into_iter().enumerate() {
        let (index, extra) = match index {
            Index::Exact(index) => (index, None),
            Index::VerbConjugation(index, polite, kind) => (
                index,
                Some(format!("Found through conjugation: {polite} / {kind:?}")),
            ),
        };

        let d = &database[index];

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

        if let Some(c) = d.as_verb_conjugation() {
            println!("# Conjugations:");

            println!("  Dictionary 終止形 (しゅうしけい) / Present / Future / Attributive:");
            println!("    {} ({})", c.dictionary.text, c.dictionary.reading);

            if let Some(form) = c.plain.get(&Conjugation::Te) {
                println!("  ~ Te:");
                println!("    {form}");
            }

            if let Some(form) = c.plain.get(&Conjugation::Negative) {
                println!("  Negative Short 未線形 (みぜんけい):");
                println!("    {form}");
            }

            println!("  Polite 連用形 (れんようけい):");

            if let Some(form) = c.polite.get(&Conjugation::Indicative) {
                println!("  ~ Present:");
                println!("    {form}");
            }

            if let Some(form) = c.polite.get(&Conjugation::Negative) {
                println!("  ~ Present Negative:");
                println!("    {form}");
            }

            if let Some(form) = c.polite.get(&Conjugation::Past) {
                println!("  ~ Past:");
                println!("    {form}");
            }

            if let Some(form) = c.polite.get(&Conjugation::PastNegative) {
                println!("  ~ Past Negative:");
                println!("    {form}");
            }

            if let Some(form) = c.plain.get(&Conjugation::Past) {
                println!("  Past:");
                println!("    {form}");
            }

            if let Some(form) = c.plain.get(&Conjugation::PastNegative) {
                println!("  Past Negative:");
                println!("    {form}");
            }

            if let Some(form) = c.plain.get(&Conjugation::Hypothetical) {
                println!("  Hypothetical / Conditional 仮定形 (かていけい):");
                println!("    {form}");
            }

            if let Some(form) = c.plain.get(&Conjugation::Conditional) {
                println!("  Conditional:");
                println!("    {form}");
            }

            if let Some(form) = c.plain.get(&Conjugation::Potential) {
                println!("  Potential 可能形 (かのうけい):");
                println!("    {form}");

                if let Some(form) = c.plain.get(&Conjugation::PotentialAlt) {
                    println!("    ~ {form} (conversational)");
                }
            }

            if let Some(form) = c.plain.get(&Conjugation::Command) {
                println!("  Command/Imperative 命令形 (めいれいけい):");
                println!("    {form}");

                if let Some(form) = c.plain.get(&Conjugation::CommandAlt) {
                    println!("    ~ {form} (alternate)");
                }
            }

            if let Some(form) = c.plain.get(&Conjugation::Volitional) {
                println!("  Volitional 意向形 (いこうけい):");
                println!("    {form}");
            }

            if let Some(form) = c.plain.get(&Conjugation::Passive) {
                println!("  Passive:");
                println!("    {form}");
            }

            if let Some(form) = c.plain.get(&Conjugation::Causative) {
                println!("  Causative:");
                println!("    {form}");
            }

            if let Some(form) = c.plain.get(&Conjugation::Tai) {
                println!("  Tai:");
                println!("    {form}");
                println!("    note: can be further conjugated as i-adjective");
            }
        }
    }

    Ok(())
}
