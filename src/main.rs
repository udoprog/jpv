mod composite;
mod elements;
mod entities;
mod parser;
mod priority;

use std::collections::HashMap;
use std::io::Read;
use std::time::Instant;

use anyhow::Result;
use elements::entry::{Conjugation, Polite};
use flate2::read::GzDecoder;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::composite::comp;

#[cfg(debug_assertions)]
fn load_dict() -> Result<String> {
    use std::fs::File;
    let input = File::open("JMdict.gz").context("JMdict.gz")?;
    let mut input = GzDecoder::new(input);
    let mut string = String::new();
    input.read_to_string(&mut string)?;
    Ok(string)
}

#[cfg(not(debug_assertions))]
fn load_dict() -> Result<String> {
    static DICT: &[u8] = include_bytes!("../JMdict.gz");
    let mut input = GzDecoder::new(std::io::Cursor::new(DICT));
    let mut string = String::new();
    input.read_to_string(&mut string)?;
    Ok(string)
}

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

    let string = load_dict()?;

    let start = Instant::now();

    let mut database = Vec::new();
    let mut lookup = HashMap::<_, Vec<Index>>::new();

    let mut parser = parser::Parser::new(&string);

    while let Some(entry) = parser.parse()? {
        tracing::trace!(?entry);

        let index = database.len();

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

    for input in std::env::args().skip(1) {
        let Some(indexes) = lookup.get(&comp([input.as_str()])) else {
            println!("nothing for `{input}`");
            continue;
        };

        for index in indexes {
            let (index, extra) = match index {
                Index::Exact(index) => (*index, None),
                Index::VerbConjugation(index, polite, kind) => {
                    (*index, Some(format!("Found through {polite} / {kind:?}")))
                }
            };

            let d = &database[index];

            if let Some(extra) = extra {
                println!("{extra}");
            }

            println!("Sequence: {}", d.sequence);

            for (index, reading) in d.reading_elements.iter().enumerate() {
                println!("  #{index} {:?}", reading.debug_sparse());
            }

            for (index, kanji) in d.kanji_elements.iter().enumerate() {
                println!("  #{index} {:?}", kanji.debug_sparse());
            }

            for (index, sense) in d.senses.iter().enumerate() {
                if !sense.is_lang("eng") {
                    continue;
                }

                println!("  #{index} {:?}", sense.debug_sparse());

                for g in &sense.gloss {
                    println!("    {:?}", g.debug_sparse());
                }
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
    }

    Ok(())
}
