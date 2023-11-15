use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use flate2::read::GzDecoder;
use lib::database;
use reqwest::Method;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::dirs::Dirs;
use crate::Args;

const USER_AGENT: &str = concat!("jpv/", env!("CARGO_PKG_VERSION"));
const JMDICT_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/JMdict_e_examp.gz";
const KANJIDIC2_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/kanjidic2.xml.gz";

#[derive(Parser)]
pub(crate) struct BuildArgs {
    /// Path to load JMDICT file from. By default this will be download into a local cache directory.
    #[arg(long, value_name = "path")]
    jmdict_path: Option<PathBuf>,
    /// Path to load kanjidic2 file from. By default this will be download into a local cache directory.
    #[arg(long, value_name = "path")]
    kanjidic_path: Option<PathBuf>,
    /// Force a dictionary rebuild.
    #[arg(long, short = 'f')]
    force: bool,
}

pub(crate) async fn run(args: &Args, build_args: &BuildArgs, dirs: &Dirs) -> Result<()> {
    let dictionary_path = match &args.dictionary {
        Some(path) => path.clone(),
        None => dirs.dictionary(),
    };

    // SAFETY: We are the only ones calling this function now.
    let result = unsafe { crate::database::load_path(&dictionary_path) };

    match result {
        Ok(data) => match database::Database::open(data) {
            Ok(..) => {
                if !build_args.force {
                    tracing::info!("Dictionary already exists at {}", dictionary_path.display());
                    return Ok(());
                } else {
                    tracing::info!(
                        "Dictionary already exists at {} (forcing rebuild)",
                        dictionary_path.display()
                    );
                }
            }
            Err(error) => {
                tracing::warn!(
                    "Rebuilding since exists, but could not open: {error}: {}",
                    dictionary_path.display()
                );
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            bail!(e)
        }
    }

    let jmdict = async {
        read_or_download(
            build_args.jmdict_path.as_deref(),
            dirs,
            "JMdict_e_examp.gz",
            JMDICT_URL,
        )
        .await
        .context("loading JMDICT")
    };

    let kanjidic2 = async {
        read_or_download(
            build_args.kanjidic_path.as_deref(),
            dirs,
            "kanjidic2.xml.gz",
            KANJIDIC2_URL,
        )
        .await
        .context("loading kanjidic2")
    };

    let ((jmdict_path, jmdict), (kanjidic2_path, kanjidic2)) = tokio::try_join!(jmdict, kanjidic2)?;

    tracing::info!("Loading JMDICT: {}", jmdict_path.display());
    tracing::info!("Loading kanjidic2: {}", kanjidic2_path.display());

    let start = Instant::now();
    let data = database::build(&jmdict, &kanjidic2)?;

    let duration = Instant::now().duration_since(start);
    tracing::info!("Took {duration:?} to build dictionary");

    tracing::info!(
        "Writing dictionary to {} ({} bytes)",
        dictionary_path.display(),
        data.len()
    );

    ensure_parent_dir(&dictionary_path).await;

    fs::write(&dictionary_path, data.as_slice())
        .await
        .with_context(|| anyhow!("{}", dictionary_path.display()))?;

    Ok(())
}

async fn read_or_download(
    path: Option<&Path>,
    dirs: &Dirs,
    name: &str,
    url: &str,
) -> Result<(PathBuf, String), anyhow::Error> {
    let (path, bytes) = match path {
        Some(path) => (path.to_owned(), fs::read(path).await?),
        None => {
            let path = dirs.cache_dir(name);

            let bytes = if !path.is_file() {
                download(url, &path)
                    .await
                    .with_context(|| anyhow!("Downloading {url} to {}", path.display()))?
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
    Ok((path, string))
}

async fn download(url: &str, path: &Path) -> Result<Vec<u8>> {
    tracing::info!("Downloading {url} to {}", path.display());

    ensure_parent_dir(path).await;

    let client = reqwest::ClientBuilder::new().build()?;

    let request = client
        .request(Method::GET, url)
        .header("User-Agent", USER_AGENT)
        .build()?;

    let mut response = client.execute(request).await?;

    let mut f = File::create(path).await?;
    let mut data = Vec::new();

    while let Some(chunk) = response.chunk().await? {
        f.write_all(chunk.as_ref()).await?;
        data.extend_from_slice(chunk.as_ref());
    }

    Ok(data)
}

async fn ensure_parent_dir(path: &Path) {
    if let Some(parent) = path.parent() {
        let is_dir = match fs::metadata(parent).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Ok(metadata) if !metadata.is_dir() => false,
            _ => true,
        };

        if !is_dir {
            let _ = fs::create_dir_all(parent).await;
        }
    }
}
