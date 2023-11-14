use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use flate2::read::GzDecoder;
use lib::database::{self};
use reqwest::Method;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::dirs::Dirs;
use crate::Args;

const USER_AGENT: &'static str = concat!("jpv/", env!("CARGO_PKG_VERSION"));
const JMDICT_URL: &'static str = "http://ftp.edrdg.org/pub/Nihongo/JMdict_e_examp.gz";
const KANJIDIC2_URL: &'static str = "http://ftp.edrdg.org/pub/Nihongo/kanjidic2.xml.gz";

#[derive(Parser)]
pub(crate) struct BuildArgs {
    /// Path to load JMDICT file from. By default this will be download into a local cache directory.
    jmdict_path: Option<PathBuf>,
    /// Path to load kanjidic2 file from. By default this will be download into a local cache directory.
    kanjidic_path: Option<PathBuf>,
}

pub(crate) async fn run(args: &Args, build_args: &BuildArgs, dirs: &Dirs) -> Result<()> {
    let database_path = match &args.dictionary {
        Some(path) => path.clone(),
        None => dirs.dictionary(),
    };

    let jmdict = read_or_download(
        build_args.jmdict_path.as_deref(),
        dirs,
        "JMdict_e_examp.gz",
        JMDICT_URL,
    )
    .await
    .context("loading JMDICT")?;

    let kanjidic2 = read_or_download(
        build_args.kanjidic_path.as_deref(),
        dirs,
        "kanjidic2.xml.gz",
        KANJIDIC2_URL,
    )
    .await
    .context("loading kanjidic2")?;

    let start = Instant::now();
    let data = database::load(&jmdict, &kanjidic2)?;

    let duration = Instant::now().duration_since(start);
    tracing::info!("Took {duration:?} to build dictionary");

    tracing::info!(
        "Writing dictionary to {} ({} bytes)",
        database_path.display(),
        data.len()
    );

    fs::write(&database_path, data.as_slice())
        .await
        .with_context(|| anyhow!("{}", database_path.display()))?;

    Ok(())
}

async fn read_or_download(
    path: Option<&Path>,
    dirs: &Dirs,
    name: &str,
    url: &str,
) -> Result<String, anyhow::Error> {
    let (path, bytes) = match path {
        Some(path) => (path.to_owned(), fs::read(path).await?),
        None => {
            let path = dirs.cache_dir(name);

            let bytes = if !path.is_file() {
                download(url, &path).await.context("downloading")?
            } else {
                fs::read(&path).await?
            };

            (path, bytes)
        }
    };

    let mut input = GzDecoder::new(&bytes[..]);
    let mut string = String::new();
    input
        .read_to_string(&mut string)
        .with_context(|| path.display().to_string())?;
    Ok(string)
}

async fn download(url: &str, path: &Path) -> Result<Vec<u8>> {
    tracing::info!("Downloading {url} to {}", path.display());

    let client = reqwest::ClientBuilder::new().build()?;

    let request = client
        .request(Method::GET, url)
        .header("User-Agent", USER_AGENT)
        .build()?;

    let mut response = client.execute(request).await?;

    let mut f = File::create(path).await?;
    let mut data = Vec::new();

    while let Some(chunk) = response.chunk().await? {
        f.write(chunk.as_ref()).await?;
        data.extend_from_slice(chunk.as_ref());
    }

    Ok(data)
}
