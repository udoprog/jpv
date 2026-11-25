use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::{Mutex as StdMutex, MutexGuard, PoisonError};
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
use tokio::sync::{oneshot, Mutex, RwLock};

use crate::reporter::EventsReporter;
use crate::system::{self, SystemEvents};
use crate::tasks::{CompletedTask, TaskCompletion, TaskName, Tasks};
use crate::Args;

#[derive(Default)]
pub(crate) struct BackgroundTasks {
    pub(crate) progress: HashMap<TaskName, system::TaskProgress>,
}

pub(crate) struct Mutable {
    config: Config,
    database: Database,
}

impl Mutable {
    /// Re-open the underlying database.
    pub(crate) fn reopen_database(&mut self, indexes: &[PathBuf], dirs: &Dirs) -> Result<()> {
        let indexes = data::open_from_args(indexes, dirs).context("Opening database files")?;
        let db =
            lib::database::Database::open(indexes, &self.config).context("Opening the database")?;
        self.database = db;
        Ok(())
    }
}

/// Configuration to install.
#[derive(Default)]
pub struct Install {
    /// Limit configurations to the specified indexes.
    pub(crate) filter: Option<Vec<String>>,
    /// Force configuration.
    pub(crate) force: bool,
}

/// Events emitted by modifying the background service.
pub enum BackgroundEvent {
    /// Save configuration file.
    SaveConfig(Config, oneshot::Sender<()>),
    /// Force a database rebuild.
    Install(Install),
    /// Cancel the task with the given id.
    Cancel(String),
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
    tasks: Arc<StdMutex<BackgroundTasks>>,
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
            mutable: Arc::new(RwLock::new(Mutable { config, database })),
            tasks: Arc::new(StdMutex::new(BackgroundTasks::default())),
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
    pub(crate) async fn update_config(&self, mut config: Config) -> Option<Config> {
        let (sender, receiver) = oneshot::channel();

        // Complement missing index formats.
        for format in IndexFormat::all() {
            if !config.indexes.contains_key(format.id()) {
                config
                    .indexes
                    .insert(format.id().to_string(), format.default_config(false));
            }
        }

        let _ = self
            .channel
            .send(BackgroundEvent::SaveConfig(config.clone(), sender));

        if receiver.await.is_err() {
            return None;
        }

        self.shared.ocr.store(config.ocr, Ordering::SeqCst);
        self.mutable.write().await.config = config.clone();
        self.system_events.send(system::Event::Refresh);
        Some(config)
    }

    /// Trigger a custom installation.
    pub(crate) fn install(&self, install_all: Install) {
        let _ = self.channel.send(BackgroundEvent::Install(install_all));
    }

    /// Cancel the running task with the given id.
    pub(crate) fn cancel_task(&self, id: String) {
        let _ = self.channel.send(BackgroundEvent::Cancel(id));
    }

    /// Progress of every running task.
    pub(crate) fn running_tasks(&self) -> Vec<system::TaskProgress> {
        self.lock_tasks().progress.values().cloned().collect()
    }

    fn lock_tasks(&self) -> MutexGuard<'_, BackgroundTasks> {
        self.tasks.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Access current configuration.
    pub(crate) async fn config(&self) -> Config {
        self.mutable.read().await.config.clone()
    }

    /// Access the database currently in use.
    pub(crate) async fn database(&self) -> Database {
        self.mutable.read().await.database.clone()
    }

    /// Mark the given task as completed.
    pub(crate) fn start_task(&self, completed: &TaskCompletion, steps: usize) {
        let Some(name) = completed.name() else {
            return;
        };

        let mut inner = self.lock_tasks();

        inner.progress.insert(
            name.clone(),
            system::TaskProgress {
                id: name.id().into(),
                name: name.to_string().into(),
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

        let Some(task) = self.lock_tasks().progress.remove(name) else {
            return;
        };

        self.system_events
            .send(system::Event::TaskCompleted(system::TaskCompleted {
                id: task.id,
                name: task.name,
                cancelled: completed.cancelled(),
            }));
    }

    /// Handle a background event.
    pub(crate) async fn handle_event(
        &self,
        event: BackgroundEvent,
        args: &Args,
        tasks: &mut Tasks,
    ) -> Result<()> {
        macro_rules! report {
            ($($result:expr),* $(,)?) => {{
                $(
                    if let Err(error) = $result {
                        tracing::error!("{error}");

                        for error in error.chain().skip(1) {
                            tracing::error!("Caused by: {error}");
                        }
                    }
                )*
            }};
        }

        match event {
            BackgroundEvent::SaveConfig(config, callback) => {
                let path = self.shared.dirs.config_path();

                let config_dir = self.shared.dirs.config_dir().to_owned();
                let new_config = config.clone();

                let task = async {
                    ensure_parent_dir(&path).await?;

                    tokio::task::spawn_blocking(move || {
                        let config = lib::toml::to_string_pretty(&config)?;

                        let mut tempfile = NamedTempFile::new_in(config_dir)?;
                        std::io::copy(&mut config.as_bytes(), &mut tempfile)?;
                        tempfile.persist(&path)?;
                        tracing::info!("Wrote new configuration to {}", path.display());
                        Ok::<_, anyhow::Error>(())
                    })
                    .await?
                };

                let task1 = task.await.context("Saving configuration to disk");

                let mut mutable = self.mutable.write().await;
                mutable.config = new_config;
                let task2 = mutable
                    .reopen_database(&args.index[..], &self.shared.dirs)
                    .context("Re-opening database");

                report!(task1, task2);
                let _ = callback.send(());
            }
            BackgroundEvent::Cancel(id) => {
                if tasks.cancel(&id) {
                    tracing::info!("Cancelling task {id}");
                } else {
                    tracing::warn!("No running task {id} to cancel");
                }
            }
            BackgroundEvent::Install(install_all) => {
                let config = self.config().await;

                let to_download = config_to_download(
                    &config,
                    &self.shared.dirs,
                    Default::default(),
                    install_all.filter.as_deref(),
                );

                let force = install_all.force;

                let mut installing = Vec::new();

                for to_download in to_download {
                    let Some((shutdown, completion)) =
                        tasks.unique_task(TaskName::Build(to_download.name.clone()))
                    else {
                        return Ok(());
                    };

                    installing.push(to_download.name.clone());

                    // Downloading and collecting glosses are two more steps.
                    let steps = if to_download.glosses.is_some() { 8 } else { 6 };
                    self.start_task(&completion, steps);

                    let mutable = self.mutable.clone();
                    let index = args.index.clone();

                    let reporter = Arc::new(EventsReporter {
                        tasks: self.tasks.clone(),
                        system_events: self.system_events.clone(),
                        name: completion.name().cloned(),
                    });

                    tokio::spawn({
                        let shared = self.shared.clone();
                        let system_events = self.system_events.clone();

                        async move {
                            // Capture the completion handler so that it is dropped with the task.
                            let mut completion = completion;
                            let token = Token::default();

                            let task1 = cancellable(
                                &token,
                                shutdown,
                                build(reporter.clone(), &token, &shared.dirs, &to_download, force),
                            )
                            .await;

                            let task1 = if token.is_set() {
                                tracing::info!("Cancelled building `{}`", to_download.name);
                                completion.set_cancelled();
                                Ok(false)
                            } else {
                                task1.context("Re-building database")
                            };

                            let mut mutable = mutable.write().await;
                            mutable.config.set_installing(&to_download.name, false);
                            let task2 = mutable
                                .reopen_database(&index[..], &shared.dirs)
                                .context("Re-opening database");

                            report!(task1, task2);
                            system_events.send(system::Event::Refresh);
                        }
                    });
                }

                if !installing.is_empty() {
                    let mut mutable = self.mutable.write().await;

                    for id in installing {
                        mutable.config.set_installing(&id, true);
                    }

                    mutable
                        .reopen_database(&args.index[..], &self.shared.dirs)
                        .context("Re-opening database")?;

                    self.system_events.send(system::Event::Refresh);
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
    /// Multilingual JMdict to merge non-English glosses from.
    pub glosses: Option<GlossSource>,
}

/// Path and url of a multilingual JMdict.
pub struct GlossSource {
    pub url: String,
    pub path: Option<Box<Path>>,
}

/// Suffix of the download override id for an index's gloss source, as in
/// `--path jmdict.glosses=JMdict.gz`.
const GLOSSES_SUFFIX: &str = ".glosses";

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
///
/// Without a `filter` only enabled indexes are included. With one, exactly the
/// named indexes are included whether enabled or not, since the config page
/// requests an install of an index before the configuration enabling it has
/// been saved.
pub fn config_to_download(
    config: &Config,
    dirs: &Dirs,
    overrides: DownloadOverrides<'_>,
    filter: Option<&[String]>,
) -> Vec<ToDownload> {
    let mut downloads = Vec::new();

    for (id, index) in &config.indexes {
        let included = match filter {
            Some(filter) => filter.contains(id),
            None => index.enabled,
        };

        if !included {
            continue;
        }

        let path = overrides.get(id.as_str()).map(|p| p.into());

        let glosses = index.gloss_url().map(|url| GlossSource {
            url: url.to_owned(),
            path: overrides
                .get(&format!("{id}{GLOSSES_SUFFIX}"))
                .map(|p| p.into()),
        });

        downloads.push(ToDownload {
            name: id.into(),
            url: index.url.clone(),
            index_path: dirs.index_path(id).into(),
            path,
            format: index.format,
            glosses,
        });
    }

    downloads
}

/// Build the database in the background.
#[must_use = "Must check that the build completed before proceeding"]
pub(crate) async fn build(
    reporter: Arc<dyn Reporter>,
    shutdown: &Token,
    dirs: &Dirs,
    download: &ToDownload,
    force: bool,
) -> Result<bool> {
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

    let (path, data) = read_or_download(
        &*reporter,
        shutdown,
        download.path.as_deref(),
        dirs,
        &download.url,
        force,
    )
    .await
    .context("Reading dictionary")?;

    tracing::info!("Loading `{}` from {}", download.name, path.display());

    let glosses = match &download.glosses {
        Some(glosses) => {
            let (path, data) = read_or_download(
                &*reporter,
                shutdown,
                glosses.path.as_deref(),
                dirs,
                &glosses.url,
                force,
            )
            .await
            .context("Reading multilingual glosses")?;

            tracing::info!(
                "Merging glosses into `{}` from {}",
                download.name,
                path.display()
            );

            Some(data)
        }
        None => None,
    };

    let start = Instant::now();
    let kind = download.format;
    let name = download.name.clone();

    ensure_running(shutdown)?;

    let buf = tokio::task::spawn_blocking({
        let reporter = reporter.clone();
        let shutdown = shutdown.clone();
        move || {
            let input = match kind {
                IndexFormat::Jmdict => Input::Jmdict {
                    jmdict: str::from_utf8(&data[..])?,
                    glosses: glosses.as_deref().map(str::from_utf8).transpose()?,
                },
                IndexFormat::Kanjidic2 => Input::Kanjidic2(str::from_utf8(&data[..])?),
                IndexFormat::Jmnedict => Input::Jmnedict(str::from_utf8(&data[..])?),
                IndexFormat::Kradfile => Input::Kradfile(&data[..]),
            };

            database::build(&*reporter, &shutdown, &name, input)
        }
    })
    .await??;

    let duration = Instant::now().duration_since(start);

    reporter.instrument_start(
        module_path!(),
        &format_args!("Saving to {}", download.index_path.display()),
        None,
    );

    ensure_running(shutdown)?;

    // Replaced in one step, so that an interrupted save leaves the previous
    // index in place rather than a truncated one.
    write_atomic(&download.index_path, buf.as_slice().to_vec())
        .await
        .with_context(|| anyhow!("{}", download.index_path.display()))?;

    tracing::info!(
        "Took {duration:?} to build index at {}",
        download.index_path.display()
    );

    reporter.instrument_end(0);
    Ok(true)
}

/// Drive `future`, setting `token` once `shutdown` is signalled so that the
/// future can wind down.
async fn cancellable<F>(token: &Token, shutdown: oneshot::Receiver<()>, future: F) -> F::Output
where
    F: std::future::Future,
{
    tokio::pin!(future);

    tokio::select! {
        output = &mut future => output,
        Ok(()) = shutdown => {
            token.set();
            future.await
        }
    }
}

fn ensure_running(shutdown: &Token) -> Result<()> {
    if shutdown.is_set() {
        bail!("Task cancelled");
    }

    Ok(())
}

async fn write_atomic<T>(path: &Path, data: T) -> Result<T>
where
    T: 'static + Send + AsRef<[u8]>,
{
    let parent = ensure_parent_dir(path).await?.to_owned();
    let path = path.to_owned();

    tokio::task::spawn_blocking(move || {
        let mut file = NamedTempFile::new_in(parent)?;
        std::io::Write::write_all(&mut file, data.as_ref())?;
        file.persist(&path)?;
        Ok(data)
    })
    .await?
}

async fn read_or_download(
    reporter: &dyn Reporter,
    shutdown: &Token,
    path: Option<&Path>,
    dirs: &Dirs,
    url: &str,
    force: bool,
) -> Result<(PathBuf, Vec<u8>), anyhow::Error> {
    let (path, bytes) = match path {
        Some(path) => (path.to_owned(), fs::read(path).await?),
        None => {
            let Some((_, name)) = url.rsplit_once('/') else {
                bail!("Url doesn't have a trailing component: {url}")
            };

            let hash = crate::hash::hash(url);
            let path = dirs.cache_dir(format!("{hash:08x}-{name}"));

            let bytes = if !path.is_file() || force {
                download(reporter, shutdown, url, &path)
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
async fn download(_: &dyn Reporter, _: &Token, _: &str, _: &Path) -> Result<Vec<u8>> {
    bail!("Downloading is not supported")
}

#[cfg(feature = "reqwest")]
async fn download(
    reporter: &dyn Reporter,
    shutdown: &Token,
    url: &str,
    path: &Path,
) -> Result<Vec<u8>> {
    use reqwest::Method;

    tracing::info!("Downloading {url} to {}", path.display());

    let client = reqwest::ClientBuilder::new().build()?;

    let request = client
        .request(Method::GET, url)
        .header("User-Agent", crate::USER_AGENT)
        .build()?;

    let mut response = client.execute(request).await?;

    let total = response
        .content_length()
        .map(|n| usize::try_from(n).unwrap_or(usize::MAX));

    let mut data = Vec::new();

    reporter.instrument_start(module_path!(), &format!("Downloading {url}"), total);

    while let Some(chunk) = response.chunk().await? {
        ensure_running(shutdown)?;
        data.extend_from_slice(chunk.as_ref());
        reporter.instrument_progress(chunk.as_ref().len());
    }

    // Only a complete download is cached, so a cancelled one is fetched
    // again rather than read back truncated.
    write_atomic(path, data).await
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

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::path::Path;
    use std::sync::Arc;

    use flate2::write::GzEncoder;
    use flate2::Compression;
    use lib::config::{Config, IndexFormat};
    use lib::reporter::EmptyReporter;
    use lib::token::Token;
    use lib::Dirs;

    use super::{build, config_to_download, DownloadOverrides, ToDownload};

    fn kradfile(dir: &Path) -> ToDownload {
        let source = dir.join("kradfile.gz");
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(b"# empty\n").unwrap();
        std::fs::write(&source, encoder.finish().unwrap()).unwrap();

        ToDownload {
            name: "kradfile".into(),
            url: String::new(),
            index_path: dir.join("index").join("kradfile.bin").into(),
            path: Some(source.into()),
            format: IndexFormat::Kradfile,
            glosses: None,
        }
    }

    #[tokio::test]
    async fn cancelled_build_keeps_existing_index() {
        let dir = tempfile::tempdir().unwrap();
        let dirs = Dirs::open().unwrap();
        let download = kradfile(dir.path());

        assert!(build(
            Arc::new(EmptyReporter),
            &Token::default(),
            &dirs,
            &download,
            false
        )
        .await
        .unwrap());
        let built = std::fs::read(&download.index_path).unwrap();

        let token = Token::default();
        token.set();

        assert!(
            build(Arc::new(EmptyReporter), &token, &dirs, &download, true)
                .await
                .is_err()
        );
        assert_eq!(std::fs::read(&download.index_path).unwrap(), built);

        let parent = download.index_path.parent().unwrap();
        assert_eq!(std::fs::read_dir(parent).unwrap().count(), 1);
    }

    fn config() -> Config {
        let mut config = Config::default();

        for format in IndexFormat::all() {
            config.indexes.insert(
                format.id().to_owned(),
                format.default_config(format == IndexFormat::Jmdict),
            );
        }

        config
    }

    fn names(config: &Config, filter: Option<&[String]>) -> Vec<String> {
        let dirs = Dirs::open().expect("dirs");
        config_to_download(config, &dirs, DownloadOverrides::default(), filter)
            .into_iter()
            .map(|d| d.name)
            .collect()
    }

    #[test]
    fn skips_disabled_without_filter() {
        assert_eq!(names(&config(), None), ["jmdict"]);
    }

    #[test]
    fn filter_includes_disabled() {
        let filter = ["kanjidic2".to_owned()];
        assert_eq!(names(&config(), Some(&filter)), ["kanjidic2"]);
    }
}
