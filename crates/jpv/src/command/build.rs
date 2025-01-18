use std::sync::Arc;

use anyhow::{bail, Result};
use clap::Parser;

use lib::config::Config;
use lib::reporter::EmptyReporter;
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
    #[arg(long, short = 'f', value_name = "name")]
    force: Vec<String>,
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

    let to_download = crate::background::config_to_download(&config, dirs, overrides, None);

    let force_all = build_args.force.first().is_some_and(|v| v == "all");

    for to_download in to_download {
        let tracing_reporter = Arc::new(EmptyReporter);
        let (_sender, shutdown) = oneshot::channel();

        crate::background::build(
            tracing_reporter,
            shutdown,
            dirs,
            &to_download,
            force_all || build_args.force.contains(&to_download.name),
        )
        .await?;
    }

    crate::dbus::shutdown().await?;
    Ok(())
}
