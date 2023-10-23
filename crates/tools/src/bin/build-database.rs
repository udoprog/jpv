use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use flate2::read::GzDecoder;
use lib::database::{self};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
struct Args {
    /// Output directory.
    #[arg(long)]
    out: Option<PathBuf>,
    /// Path to load dictionary from. Defaults to `JMdict_e_examp.gz`.
    path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let filter = EnvFilter::builder().from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let args = Args::try_parse()?;

    let database_path = args
        .out
        .as_deref()
        .unwrap_or(Path::new("."))
        .join("database.bin");

    let jmdict = args
        .path
        .as_deref()
        .unwrap_or(Path::new("JMdict_e_examp.gz"));

    let kanjidic2 = args
        .path
        .as_deref()
        .unwrap_or(Path::new("kanjidic2.xml.gz"));

    let start = Instant::now();

    let jmdict = load_file(jmdict).with_context(|| jmdict.display().to_string())?;
    let kanjidic2 = load_file(kanjidic2).with_context(|| kanjidic2.display().to_string())?;
    let data = database::load(&jmdict, &kanjidic2)?;

    let duration = Instant::now().duration_since(start);
    tracing::info!(?duration);

    fs::write(&database_path, data.as_slice())
        .with_context(|| anyhow!("{}", database_path.display()))?;
    Ok(())
}

fn load_file(path: &Path) -> Result<String> {
    let input = File::open(path)?;
    let mut input = GzDecoder::new(input);
    let mut string = String::new();
    input.read_to_string(&mut string)?;
    Ok(string)
}
