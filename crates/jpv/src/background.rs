use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use anyhow::{anyhow, bail, Context, Result};
use flate2::read::GzDecoder;
use lib::config::{Config, IndexFormat};
use lib::database::{self, Database, Input};
use lib::reporter::Reporter;
use lib::token::Token;
use lib::{api, data, Dirs};
use tempfile::NamedTempFile;
use tokio::fs;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{oneshot, Mutex};

use crate::reporter::EventsReporter;
use crate::system::{self, SystemEvents};
use crate::tasks::{CompletedTask, TaskCompletion, Tasks};
use crate::Args;

pub(crate) struct Mutable {
    config: Config,
    database: Database,
    pub(crate) tasks: HashMap<Box<str>, system::TaskProgress>,
}

/// Events emitted by modifying the background service.
pub enum BackgroundEvent {
    /// Save configuration file.
    SaveConfig(Config, oneshot::Sender<()>),
    /// Force a database rebuild.
    InstallAll(bool),
}

struct Shared {
    dirs: Dirs,
    tesseract: Option<Mutex<tesseract::Tesseract>>,
    ocr: AtomicBool,
}

#[derive(Clone)]
pub struct Background {
    shared: Arc<Shared>,
    channel: UnboundedSender<BackgroundEvent>,
    system_events: SystemEvents,
    mutable: Arc<RwLock<Mutable>>,
    log: crate::log::Capture,
}

impl Background {
    pub(crate) fn new(
        dirs: Dirs,
        channel: UnboundedSender<BackgroundEvent>,
        config: Config,
        database: Database,
        system_events: SystemEvents,
        tesseract: Option<tesseract::Tesseract>,
        log: crate::log::Capture,
    ) -> Result<Self> {
        let tesseract = tesseract.map(Mutex::new);

        Ok(Self {
            shared: Arc::new(Shared {
                dirs,
                tesseract,
                ocr: AtomicBool::new(config.ocr),
            }),
            channel,
            system_events,
            mutable: Arc::new(RwLock::new(Mutable {
                config,
                database,
                tasks: HashMap::new(),
            })),
            log,
        })
    }

    /// Get tesseract API handle.
    pub(crate) fn tesseract(&self) -> Option<&Mutex<tesseract::Tesseract>> {
        if !self.shared.ocr.load(Ordering::SeqCst) {
            return None;
        }

        self.shared.tesseract.as_ref()
    }

    /// Get the current log backfill.
    pub(crate) fn log(&self) -> Vec<api::OwnedLogEntry> {
        self.log.read()
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

        self.shared.ocr.store(config.ocr, Ordering::SeqCst);
        self.mutable.write().unwrap().config = config;
        self.system_events.send(system::Event::Refresh);
        true
    }

    /// Trigger a rebuild.
    pub(crate) async fn rebuild(&self) {
        let _ = self.channel.send(BackgroundEvent::InstallAll(false));
    }

    /// Access current configuration.
    pub(crate) fn config(&self) -> Config {
        self.mutable.read().unwrap().config.clone()
    }

    /// Access the database currently in use.
    pub(crate) fn database(&self) -> Database {
        self.mutable.read().unwrap().database.clone()
    }

    /// Mark the given task as completed.
    pub(crate) fn start_task(&self, completed: &TaskCompletion, steps: usize) {
        let Some(name) = completed.name() else {
            return;
        };

        let mut inner = self.mutable.write().unwrap();

        inner.tasks.insert(
            name.into(),
            system::TaskProgress {
                name: name.into(),
                value: 0,
                total: None,
                text: String::new(),
                step: 0,
                steps,
            },
        );
    }

    /// Mark the given task as completed.
    pub(crate) fn complete_task(&self, completed: CompletedTask) {
        let Some(name) = completed.name() else {
            return;
        };

        let mut inner = self.mutable.write().unwrap();

        let Some(task) = inner.tasks.remove(name) else {
            return;
        };

        self.system_events
            .send(system::Event::TaskCompleted(system::TaskCompleted {
                name: task.name,
            }));
    }

    /// Handle a background event.
    pub(crate) async fn handle_event(
        &self,
        event: BackgroundEvent,
        args: &Args,
        tasks: &mut Tasks,
    ) -> Result<()> {
        match event {
            BackgroundEvent::SaveConfig(config, callback) => {
                let path = self.shared.dirs.config_path();
                ensure_parent_dir(&path).await?;

                let config_dir = self.shared.dirs.config_dir().to_owned();
                let new_config = config.clone();

                let task = tokio::task::spawn_blocking(move || {
                    let config = lib::toml::to_string_pretty(&config)?;

                    let mut tempfile = NamedTempFile::new_in(config_dir)?;
                    std::io::copy(&mut config.as_bytes(), &mut tempfile)?;
                    tempfile.persist(&path)?;
                    tracing::info!("Wrote new configuration to {}", path.display());
                    Ok::<_, anyhow::Error>(())
                });

                task.await??;

                let indexes = data::open_from_args(&args.index[..], &self.shared.dirs)
                    .context("Opening database files")?;
                let db = lib::database::Database::open(indexes, &new_config)
                    .context("Opening the database")?;
                self.mutable.write().unwrap().database = db;
                let _ = callback.send(());
            }
            BackgroundEvent::InstallAll(force) => {
                let config = self.config();
                let to_download =
                    config_to_download(&config, &self.shared.dirs, Default::default());

                for to_download in to_download {
                    let Some((shutdown, completion)) =
                        tasks.unique_task(format!("Building {}", to_download.name))
                    else {
                        return Ok(());
                    };

                    self.start_task(&completion, 6);

                    let inner = self.mutable.clone();
                    let index = args.index.clone();

                    let reporter = Arc::new(EventsReporter {
                        inner: inner.clone(),
                        system_events: self.system_events.clone(),
                        name: completion.name().map(Box::from),
                    });

                    tokio::spawn({
                        let immutable = self.shared.clone();
                        let system_events = self.system_events.clone();

                        async move {
                            // Capture the completion handler so that it is dropped with the task.
                            let _completion = completion;

                            let future = async {
                                if !build(
                                    reporter.clone(),
                                    shutdown,
                                    &immutable.dirs,
                                    &to_download,
                                    force,
                                )
                                .await?
                                {
                                    return Ok(());
                                }

                                let indexes = data::open_from_args(&index[..], &immutable.dirs)?;
                                let mut inner = inner.write().unwrap();
                                let db = lib::database::Database::open(indexes, &inner.config)?;
                                inner.database = db;
                                Ok::<_, anyhow::Error>(())
                            };

                            if let Err(error) = future.await {
                                tracing::error!("Failed to build index");

                                for error in error.chain() {
                                    tracing::error!("Caused by: {error}");
                                }
                            }

                            system_events.send(system::Event::Refresh);
                        }
                    });
                }
            }
        }

        Ok(())
    }
}

/// Path and url to download.
pub struct ToDownload {
    pub name: String,
    pub url: String,
    pub index_path: Box<Path>,
    pub path: Option<Box<Path>>,
    pub format: IndexFormat,
}

/// Download override paths.
#[derive(Default)]
pub struct DownloadOverrides<'a> {
    overrides: HashMap<&'a str, &'a Path>,
}

impl<'a> DownloadOverrides<'a> {
    /// Insert a download override.
    pub fn insert<P>(&mut self, id: &'a str, path: &'a P)
    where
        P: ?Sized + AsRef<Path>,
    {
        self.overrides.insert(id, path.as_ref());
    }

    /// Get an individual override.
    fn get(&self, id: &str) -> Option<&'a Path> {
        self.overrides.get(id).copied()
    }
}

/// Convert configuration into indexes that should be downloaded and built.
pub fn config_to_download(
    config: &Config,
    dirs: &Dirs,
    overrides: DownloadOverrides<'_>,
) -> Vec<ToDownload> {
    let mut downloads = Vec::new();

    for (id, index) in &config.indexes {
        let path = overrides.get(id.as_str()).map(|p| p.into());

        downloads.push(ToDownload {
            name: id.into(),
            url: index.url.clone(),
            index_path: dirs.index_path(id).into(),
            path,
            format: index.format,
        });
    }

    downloads
}

/// Build the database in the background.
#[must_use = "Must check that the build completed before proceeding"]
pub(crate) async fn build(
    reporter: Arc<dyn Reporter>,
    shutdown: oneshot::Receiver<()>,
    dirs: &Dirs,
    download: &ToDownload,
    force: bool,
) -> Result<bool> {
    let shutdown_token = Token::default();
    ensure_parent_dir(&download.index_path).await?;

    // SAFETY: We are the only ones calling this function now.
    let result = lib::data::open(&download.index_path);

    match result {
        Ok(data) => match database::Index::open(data) {
            Ok(..) => {
                if !force {
                    tracing::info!(
                        "Dictionary already exists at {}",
                        download.index_path.display()
                    );
                    return Ok(false);
                } else {
                    tracing::info!(
                        "Dictionary already exists at {} (forcing rebuild)",
                        download.index_path.display()
                    );
                }
            }
            Err(error) => {
                tracing::warn!(
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

    let (path, data) = read_or_download(&*reporter, download.path.as_deref(), dirs, &download.url)
        .await
        .context("Reading dictionary")?;

    tracing::info!("Loading `{}` from {}", download.name, path.display());

    let start = Instant::now();
    let kind = download.format;
    let name = download.name.clone();

    let mut task = tokio::task::spawn_blocking({
        let reporter = reporter.clone();
        let shutdown_token = shutdown_token.clone();
        move || {
            let input = match kind {
                IndexFormat::Jmdict => Input::Jmdict(str::from_utf8(&data[..])?),
                IndexFormat::Kanjidic2 => Input::Kanjidic2(str::from_utf8(&data[..])?),
                IndexFormat::Jmnedict => Input::Jmnedict(str::from_utf8(&data[..])?),
                IndexFormat::Kradfile => Input::Kradfile(&data[..]),
            };

            database::build(&*reporter, &shutdown_token, &name, input)
        }
    });

    let buf = tokio::select! {
        result = &mut task => {
            result??
        }
        _ = shutdown => {
            shutdown_token.set();
            task.await??
        }
    };

    let duration = Instant::now().duration_since(start);

    reporter.instrument_start(
        module_path!(),
        &format_args!("Saving to {}", download.index_path.display()),
        None,
    );

    fs::write(&download.index_path, buf.as_slice())
        .await
        .with_context(|| anyhow!("{}", download.index_path.display()))?;

    tracing::info!(
        "Took {duration:?} to build index at {}",
        download.index_path.display()
    );

    reporter.instrument_end(0);
    Ok(true)
}

async fn read_or_download(
    reporter: &dyn Reporter,
    path: Option<&Path>,
    dirs: &Dirs,
    url: &str,
) -> Result<(PathBuf, Vec<u8>), anyhow::Error> {
    let (path, bytes) = match path {
        Some(path) => (path.to_owned(), fs::read(path).await?),
        None => {
            let Some((_, name)) = url.rsplit_once('/') else {
                bail!("Url doesn't have a trailing component: {url}")
            };

            let hash = crate::hash::hash(url);
            let path = dirs.cache_dir(format!("{hash:08x}-{name}"));

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

    reporter.instrument_end(bytes.len());

    let mut input = GzDecoder::new(&bytes[..]);
    let mut bytes = Vec::new();
    input
        .read_to_end(&mut bytes)
        .with_context(|| path.display().to_string())?;
    Ok((path, bytes))
}

#[cfg(not(feature = "reqwest"))]
async fn download(_: &dyn Reporter, _: &str, _: &Path) -> Result<Vec<u8>> {
    bail!("Downloading is not supported")
}

#[cfg(feature = "reqwest")]
async fn download(reporter: &dyn Reporter, url: &str, path: &Path) -> Result<Vec<u8>> {
    use reqwest::Method;
    use tokio::io::AsyncWriteExt;

    tracing::info!("Downloading {url} to {}", path.display());

    ensure_parent_dir(path).await?;

    let client = reqwest::ClientBuilder::new().build()?;

    let request = client
        .request(Method::GET, url)
        .header("User-Agent", crate::USER_AGENT)
        .build()?;

    let mut response = client.execute(request).await?;

    let total = response
        .content_length()
        .map(|n| usize::try_from(n).unwrap_or(usize::MAX));

    let mut f = fs::File::create(path).await?;
    let mut data = Vec::new();

    reporter.instrument_start(module_path!(), &format!("Downloading {url}"), total);

    while let Some(chunk) = response.chunk().await? {
        f.write_all(chunk.as_ref()).await?;
        data.extend_from_slice(chunk.as_ref());
        reporter.instrument_progress(chunk.as_ref().len());
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
