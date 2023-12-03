use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;

use lib::config::{Config, IndexKind};
use lib::reporter::TracingReporter;
use lib::Dirs;
use tokio::sync::oneshot;

use crate::background::DownloadOverrides;
use crate::Args;

#[derive(Parser)]
pub(crate) struct BuildArgs {
    /// Path to load JMDICT file from. By default this will be download into a local cache directory.
    #[arg(long, value_name = "path")]
    jmdict_path: Option<PathBuf>,
    /// Path to load kanjidic2 file from. By default this will be download into a local cache directory.
    #[arg(long, value_name = "path")]
    kanjidic2_path: Option<PathBuf>,
    /// Path to load jmnedict file from. By default this will be download into a local cache directory.
    #[arg(long, value_name = "path")]
    jmnedict_path: Option<PathBuf>,
    /// Force a dictionary rebuild.
    #[arg(long, short = 'f')]
    force: bool,
}

pub(crate) async fn run(
    _: &Args,
    build_args: &BuildArgs,
    dirs: &Dirs,
    config: Config,
) -> Result<()> {
    let mut overrides = DownloadOverrides::default();

    if let Some(path) = &build_args.jmdict_path {
        overrides.insert(IndexKind::Jmdict, path);
    }

    if let Some(path) = &build_args.kanjidic2_path {
        overrides.insert(IndexKind::Kanjidic2, path);
    }

    if let Some(path) = &build_args.jmnedict_path {
        overrides.insert(IndexKind::Jmnedict, path);
    }

    let to_download = crate::background::config_to_download(&config, dirs, overrides);

    for to_download in to_download {
        let tracing_reporter = Arc::new(TracingReporter);
        let (_sender, shutdown) = oneshot::channel();

        crate::background::build(
            tracing_reporter,
            shutdown,
            dirs,
            &to_download,
            build_args.force,
        )
        .await?;
    }

    crate::dbus::shutdown().await?;
    Ok(())
}
