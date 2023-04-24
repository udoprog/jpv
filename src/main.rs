mod elements;
mod entities;
mod parser;
mod priority;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::time::Instant;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::elements::entry::VerbKind;

fn main() -> Result<()> {
    let filter = EnvFilter::builder().from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let input = File::open("JMdict.gz").context("JMdict.gz")?;
    let mut input = GzDecoder::new(input);
    let mut string = String::new();
    input.read_to_string(&mut string)?;

    let start = Instant::now();

    let mut database = Vec::new();
    let mut lookup = HashMap::<_, Vec<usize>>::new();

    let mut parser = parser::Parser::new(&string);

    while let Some(entry) = parser.parse()? {
        tracing::trace!(?entry);

        let index = database.len();

        for el in &entry.reading_elements {
            lookup.entry(el.text).or_default().push(index);
        }

        for el in &entry.kanji_elements {
            lookup.entry(el.text).or_default().push(index);
        }

        database.push(entry);
    }

    let duration = Instant::now().duration_since(start);
    tracing::info!(?duration);

    for input in std::env::args().skip(1) {
        let Some(indexes) = lookup.get(input.as_str()) else {
            println!("nothing for `{input}`");
            continue;
        };

        for &index in indexes {
            let d = &database[index];

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

            let (Some(verb), [kanji, ..], [reading, ..]) = (d.as_verb_kind(), &d.kanji_elements[..], &d.reading_elements[..]) else {
                continue;
            };

            match verb {
                VerbKind::Ichidan => {
                    let mut k = kanji.text.chars();
                    let mut r = reading.text.chars();

                    match (k.next_back(), r.next_back()) {
                        (Some('る'), Some('る')) => {}
                        _ => {
                            println!("Don't know how to conjugate it");
                            continue;
                        }
                    }

                    let stem = k.as_str();
                    let stem_reading = r.as_str();

                    println!("# Conjugations:");
                    println!("  Stem: {stem} ({})", stem_reading);

                    println!("  Negative Short 未線形 (みぜんけい):");
                    println!("    {stem}ない");
                    println!("  Polite 連用形 (れんようけい):");
                    println!("  ~ Present:");
                    println!("    {stem}ます");
                    println!("  ~ Present Negative:");
                    println!("    {stem}ません");
                    println!("  ~ Past:");
                    println!("    {stem}ました");
                    println!("  ~ Past Negative:");
                    println!("    {stem}ませんでした");
                    println!(
                        "  Dictionary 終止形 (しゅうしけい) / Present / Future / Attributive:"
                    );
                    println!("    {} ({})", kanji.text, reading.text);
                    println!("  Hypothetical / Conditional Form 仮定形 (かていけい):");
                    println!("    {stem}れば");
                    println!("  Potential Form 可能形 (かのうけい):");
                    println!("    {stem}られる (conversational: {stem}れる)");
                    println!("  Command/Imperative Form 命令形 (めいれいけい):");
                    println!("    {stem}ろ (alt: {stem}よ)");
                    println!("  Volitional Form 意向形 (いこうけい):");
                    println!("    {stem}よう");
                }
                VerbKind::Godan => {
                    let mut k = kanji.text.chars();
                    let mut r = reading.text.chars();

                    let [a, i, e, o] = match (k.next_back(), r.next_back()) {
                        (Some(a @ 'う'), Some(b)) if a == b => ['わ', 'い', 'え', 'お'],
                        (Some(a @ 'く'), Some(b)) if a == b => ['か', 'き', 'け', 'こ'],
                        (Some(a @ 'つ'), Some(b)) if a == b => ['た', 'ち', 'て', 'と'],
                        (Some(a @ 'ぐ'), Some(b)) if a == b => ['が', 'ぎ', 'げ', 'ご'],
                        (Some(a @ 'む'), Some(b)) if a == b => ['ま', 'み', 'め', 'も'],
                        (Some(a @ 'す'), Some(b)) if a == b => ['さ', 'し', 'せ', 'そ'],
                        (Some(a @ 'ぶ'), Some(b)) if a == b => ['ば', 'び', 'べ', 'ぼ'],
                        (Some(a @ 'ぬ'), Some(b)) if a == b => ['な', 'に', 'ね', 'の'],
                        (Some(a @ 'る'), Some(b)) if a == b => ['ら', 'り', 'れ', 'ろ'],
                        _ => {
                            println!("Don't know how to conjugate it");
                            continue;
                        }
                    };

                    let stem = k.as_str();
                    let stem_reading = r.as_str();

                    println!("# Conjugations:");
                    println!("  Stem: {stem} ({stem_reading})");

                    println!("  Negative Short 未線形 (みぜんけい):");
                    println!("    {stem}{a}ない");
                    println!("  Polite 連用形 (れんようけい):");
                    println!("  ~ Present:");
                    println!("    {stem}{i}ます");
                    println!("  ~ Present Negative:");
                    println!("    {stem}{i}ません");
                    println!("  ~ Past:");
                    println!("    {stem}{i}ました");
                    println!("  ~ Past Negative:");
                    println!("    {stem}{i}ませんでした");
                    println!(
                        "  Dictionary 終止形 (しゅうしけい) / Present / Future / Attributive:"
                    );
                    println!("    {} ({})", kanji.text, reading.text);
                    println!("  Hypothetical / Conditional Form 仮定形 (かていけい):");
                    println!("    {stem}{e}ば");
                    println!("  Potential Form 可能形 (かのうけい):");
                    println!("    {stem}{e}る (conversational: {stem}{e}る)");
                    println!("  Command/Imperative Form 命令形 (めいれいけい):");
                    println!("    {stem}{e}");
                    println!("  Volitional Form 意向形 (いこうけい):");
                    println!("    {stem}{o}う");
                }
            }
        }
    }

    Ok(())
}
