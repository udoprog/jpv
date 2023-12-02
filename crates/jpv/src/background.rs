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
use lib::{data, Dirs};
use reqwest::Method;
use tempfile::NamedTempFile;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use crate::Args;

/// The user agent used by jpv.
const USER_AGENT: &str = concat!("jpv/", env!("CARGO_PKG_VERSION"));

const JMDICT_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/JMdict_e_examp.gz";
const KANJIDIC2_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/kanjidic2.xml.gz";
const JMNEDICT_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/JMnedict.xml.gz";

struct Inner {
    config: Config,
    database: Database,
}

/// Events emitted by modifying the background service.
pub enum BackgroundEvent {
    SaveConfig(Config, oneshot::Sender<()>),
}

#[derive(Clone)]
pub struct Background {
    dirs: Arc<Dirs>,
    channel: UnboundedSender<BackgroundEvent>,
    inner: Arc<RwLock<Inner>>,
}

impl Background {
    pub(crate) fn new(
        dirs: Dirs,
        channel: UnboundedSender<BackgroundEvent>,
        config: Config,
        database: Database,
    ) -> Self {
        Self {
            dirs: Arc::new(dirs),
            channel,
            inner: Arc::new(RwLock::new(Inner { config, database })),
        }
    }

    /// Update current configuration.
    pub(crate) async fn update_config(&self, config: Config) -> bool {
        let (sender, receiver) = oneshot::channel();

        let _ = self
            .channel
            .send(BackgroundEvent::SaveConfig(config.clone(), sender));

        if receiver.await.is_err() {
            return false;
        }

        self.inner.write().unwrap().config = config;
        true
    }

    /// Access current configuration.
    pub(crate) fn config(&self) -> Config {
        self.inner.read().unwrap().config.clone()
    }

    /// Access the database currently in use.
    pub(crate) fn database(&self) -> Database {
        self.inner.read().unwrap().database.clone()
    }

    /// Handle a background event.
    pub(crate) async fn handle_event(&self, event: BackgroundEvent, args: &Args) -> Result<()> {
        match event {
            BackgroundEvent::SaveConfig(config, callback) => {
                let dirs = self.dirs.clone();
                let path = dirs.config_path();
                ensure_parent_dir(&path).await?;

                let new_config = config.clone();

                let task = tokio::task::spawn_blocking(move || {
                    let config = lib::toml::to_string_pretty(&config)?;

                    let mut tempfile = NamedTempFile::new_in(dirs.config_dir())?;
                    std::io::copy(&mut config.as_bytes(), &mut tempfile)?;
                    tempfile.persist(&path)?;
                    tracing::info!("Wrote new configuration to {}", path.display());
                    Ok::<_, anyhow::Error>(())
                });

                task.await??;

                let indexes = data::open_from_args(&args.index[..], &self.dirs)?;
                let db = lib::database::Database::open(indexes, &new_config)?;
                self.inner.write().unwrap().database = db;
                let _ = callback.send(());
            }
        }

        Ok(())
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

    for &index in &config.enabled {
        let path = overrides.get(index).map(|p| p.into());

        let download = match index {
            IndexKind::Jmdict => ToDownload {
                name: index.name().into(),
                url: JMDICT_URL.into(),
                url_name: "JMdict_e_examp.gz".into(),
                index_path: dirs.index_path(index.name()).into(),
                path,
                kind: index,
            },
            IndexKind::Kanjidic2 => ToDownload {
                name: index.name().into(),
                url: KANJIDIC2_URL.into(),
                url_name: "kanjidic2.xml.gz".into(),
                index_path: dirs.index_path(index.name()).into(),
                path,
                kind: index,
            },
            IndexKind::Jmnedict => ToDownload {
                name: index.name().into(),
                url: JMNEDICT_URL.into(),
                url_name: "jmnedict.xml.gz".into(),
                index_path: dirs.index_path(index.name()).into(),
                path,
                kind: index,
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
        ensure_parent_dir(&download.index_path).await?;

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

    ensure_parent_dir(path).await?;

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

async fn ensure_parent_dir(path: &Path) -> Result<&Path> {
    let Some(parent) = path.parent() else {
        bail!("Missing parent directory for {}", path.display());
    };

    let is_dir = match fs::metadata(parent).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Ok(metadata) if !metadata.is_dir() => false,
        _ => true,
    };

    if !is_dir {
        fs::create_dir_all(parent).await?;
    }

    Ok(parent)
}
