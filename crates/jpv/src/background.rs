use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use anyhow::{anyhow, bail, Context, Result};
use flate2::read::GzDecoder;
use lib::config::{Config, IndexKind};
use lib::database::{self, Database, Input};
use lib::reporter::Reporter;
use lib::Dirs;
use reqwest::Method;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// The user agent used by jpv.
const USER_AGENT: &str = concat!("jpv/", env!("CARGO_PKG_VERSION"));

const JMDICT_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/JMdict_e_examp.gz";
const KANJIDIC2_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/kanjidic2.xml.gz";
const JMNEDICT_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/JMnedict.xml.gz";

struct Inner {
    config: Config,
    database: Database,
}

#[derive(Clone)]
pub struct Background {
    dirs: Arc<Dirs>,
    inner: Arc<RwLock<Inner>>,
}

impl Background {
    pub(crate) fn new(config: Config, dirs: Dirs, database: Database) -> Self {
        Self {
            dirs: Arc::new(dirs),
            inner: Arc::new(RwLock::new(Inner { config, database })),
        }
    }

    /// Update current configuration.
    pub(crate) fn update_config(&self, config: Config) {
        self.inner.write().unwrap().config = config;
    }

    /// Access current configuration.
    pub(crate) fn config(&self) -> Config {
        self.inner.read().unwrap().config.clone()
    }

    /// Access the database currently in use.
    pub(crate) fn database(&self) -> Database {
        self.inner.read().unwrap().database.clone()
    }
}

/// Path and url to download.
pub struct ToDownload {
    pub name: String,
    pub url: String,
    pub url_name: String,
    pub index_path: Box<Path>,
    pub path: Option<Box<Path>>,
    pub kind: IndexKind,
}

/// Download override paths.
#[derive(Default)]
pub struct DownloadOverrides<'a> {
    overrides: HashMap<IndexKind, &'a Path>,
}

impl<'a> DownloadOverrides<'a> {
    /// Insert a download override.
    pub fn insert(&mut self, kind: IndexKind, path: &'a Path) {
        self.overrides.insert(kind, path);
    }

    fn get(&self, kind: IndexKind) -> Option<&'a Path> {
        self.overrides.get(&kind).copied()
    }
}

/// Convert configuration into indexes that should be downloaded and built.
pub fn config_to_download(
    config: &Config,
    dirs: &Dirs,
    overrides: DownloadOverrides<'_>,
) -> Vec<ToDownload> {
    let mut downloads = Vec::new();

    for index in &config.indexes {
        let path = overrides.get(index.kind).map(|p| p.into());

        let download = match index.kind {
            IndexKind::Jmdict => ToDownload {
                name: index.kind.name().into(),
                url: JMDICT_URL.into(),
                url_name: "JMdict_e_examp.gz".into(),
                index_path: dirs.index_path(index.kind.name()).into(),
                path,
                kind: index.kind,
            },
            IndexKind::Kanjidic2 => ToDownload {
                name: index.kind.name().into(),
                url: KANJIDIC2_URL.into(),
                url_name: "kanjidic2.xml.gz".into(),
                index_path: dirs.index_path(index.kind.name()).into(),
                path,
                kind: index.kind,
            },
            IndexKind::Jmnedict => ToDownload {
                name: index.kind.name().into(),
                url: JMNEDICT_URL.into(),
                url_name: "jmnedict.xml.gz".into(),
                index_path: dirs.index_path(index.kind.name()).into(),
                path,
                kind: index.kind,
            },
        };

        downloads.push(download);
    }

    downloads
}

/// Build the database in the background.
pub(crate) async fn build(
    reporter: &dyn Reporter,
    dirs: &Dirs,
    to_download: Vec<ToDownload>,
    force: bool,
) -> Result<()> {
    for download in &to_download {
        ensure_parent_dir(&download.index_path).await;

        // SAFETY: We are the only ones calling this function now.
        let result = lib::data::open(&download.index_path);

        match result {
            Ok(data) => match database::Index::open(data) {
                Ok(..) => {
                    if !force {
                        lib::report_info!(
                            reporter,
                            "Dictionary already exists at {}",
                            download.index_path.display()
                        );
                        continue;
                    } else {
                        lib::report_info!(
                            reporter,
                            "Dictionary already exists at {} (forcing rebuild)",
                            download.index_path.display()
                        );
                    }
                }
                Err(error) => {
                    lib::report_warn!(
                        reporter,
                        "Rebuilding since exists, but could not open: {error}: {}",
                        download.index_path.display()
                    );
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                bail!(e)
            }
        }

        let future = async {
            let (path, data) = read_or_download(
                reporter,
                download.path.as_deref(),
                dirs,
                &download.url_name,
                &download.url,
            )
            .await
            .context("loading JMDICT")?;

            lib::report_info!(
                reporter,
                "Loading `{}` from {}",
                download.name,
                path.display()
            );

            let input = match download.kind {
                IndexKind::Jmdict => Input::Jmdict(&data[..]),
                IndexKind::Kanjidic2 => Input::Kanjidic(&data[..]),
                IndexKind::Jmnedict => Input::Jmnedict(&data[..]),
            };

            let start = Instant::now();
            let data = database::build(reporter, &download.name, input)?;
            let duration = Instant::now().duration_since(start);

            fs::write(&download.index_path, data.as_slice())
                .await
                .with_context(|| anyhow!("{}", download.index_path.display()))?;

            lib::report_info!(
                reporter,
                "Took {duration:?} to build index at {}",
                download.index_path.display()
            );

            Ok::<_, anyhow::Error>(())
        };

        if let Err(error) = future.await {
            lib::report_error!(reporter, "Error building {}: {}", download.name, error);
        }
    }

    Ok(())
}

async fn read_or_download(
    reporter: &dyn Reporter,
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
                download(reporter, url, &path)
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

async fn download(reporter: &dyn Reporter, url: &str, path: &Path) -> Result<Vec<u8>> {
    lib::report_info!(reporter, "Downloading {url} to {}", path.display());

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
