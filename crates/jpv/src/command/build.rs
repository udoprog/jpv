use std::sync::Arc;

use anyhow::{bail, Result};
use clap::Parser;

use lib::config::Config;
use lib::reporter::TracingReporter;
use lib::Dirs;
use tokio::sync::oneshot;

use crate::background::DownloadOverrides;
use crate::Args;

#[derive(Parser)]
pub(crate) struct BuildArgs {
    /// Override the path to the index with the specified id and path.
    /// This takes the form `<id>=<path>`.
    #[arg(long, value_name = "path")]
    path: Vec<String>,
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

    for path in &build_args.path {
        let Some((id, path)) = path.split_once('=') else {
            bail!("Bad override: {path}");
        };

        overrides.insert(id, path);
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
