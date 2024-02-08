//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/jpv-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/jpv)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/jpv.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/jpv)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-jpv-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/jpv)
//!
//! <a href="https://github.com/udoprog/jpv">
//! <img height="128" width="128" alt="Japanese Dictionary by John-John Tedro" src="https://github.com/udoprog/jpv/blob/main/gfx/logo.png?raw=true" />
//! </a>
//!
//! Welcome to my Japanese dictionary project!
//!
//! This used to be a personal project of mine, but I have now spent enough time and
//! effort on it that I think it might be useful for others.
//!
//! <br>
//!
//! ## Overview
//!
//! <table>
//! <tr>
//! <td valign="top">
//!   <img alt="Searching for english text" src="https://github.com/udoprog/jpv/blob/main/gfx/feature-english.png?raw=true" />
//!   <div style="font-size: 0.8em;">Search for Japanese words and phrases or English glossary.</div>
//! </td>
//! <td valign="top">
//!   <img alt="Conjugations" src="https://github.com/udoprog/jpv/blob/main/gfx/feature-conjugate.png?raw=true" /><br>
//!   <div style="font-size: 0.8em;">Advanced word conjugator.</div>
//! </td>
//! </tr>
//!
//! <tr>
//! <td valign="top">
//!   <img alt="Image recognition using tesseract through the clipboard" src="https://github.com/udoprog/jpv/blob/main/gfx/feature-ocr.png?raw=true" />
//!   <div style="font-size: 0.8em;">Image recognition through the clipboard using <a href="https://github.com/tesseract-ocr/tesseract">tesseract</a> (<code>ocr</code> feature).</div>
//! </td>
//! <td valign="top">
//!   <img alt="Wildcard searching" src="https://github.com/udoprog/jpv/blob/main/gfx/feature-wildcard.png?raw=true" />
//!   <div style="font-size: 0.8em;">Wildcard searching.</div>
//! </td>
//! </tr>
//! </table>
//!
//! <br>
//!
//! ## Building and Installing
//!
//! Install dependencies for the platform you intend to build for:
//!
//! * For the `ocr` feature:
//!   * `Fedora` - `sudo dnf install tesseract-devel`
//!
//! Install [`trunk`] and the `wasm32` toolchain to build the UI:
//!
//! [`trunk`]: https://trunkrs.dev/
//!
//! ```sh
//! cargo install trunk
//! rustup target add wasm32-unknown-unknown
//! ```
//!
//! <br>
//!
//! #### Fedora
//!
//! ```
//! sudo dnf install openssl-devel git
//! ```
//!
//! For the `ocr` feature:
//!
//! ```
//! sudo dnf install gcc tesseract-devel leptonica-devel clang-devel
//! ```
//!
//! For the `gnome` feature, we make use of `gio` in order to launch a browser
//! window which needs `glib2`:
//!
//! ```
//! sudo dnf install glib2-devel
//! ```
//!
//! <br>
//!
//! #### Windows
//!
//! The OCR feature depends on tesseract which is dynamically loaded.
//!
//! You'll have to install tesseract from the [UB-Mannheim/tesseract] project.
//!
//! [UB-Mannheim/tesseract]: https://github.com/UB-Mannheim/tesseract/wiki
//!
//! Make sure to install the "Japanese" and "Japanese (vertical)" under
//! `Additional language data`.
//!
//! ![Additional language data](https://github.com/udoprog/jpv/blob/main/gfx/tesseract-ocr-japanese.png?raw=true)
//!
//! <br>
//!
//! #### Building the project
//!
//! After this, you can run the project directly in the project directory:
//!
//! ```sh
//! trunk build --release
//! cargo run --features bundle
//! ```
//!
//! > **Note:** On Linux, you probably want to include the `gnome` feature for
//! > desktop integration.
//!
//! There are scripts available to conveniently build and install packages for
//! specific environments:
//!
//! * [tools/install-fedora] to build and install for GNOME on Fedora.
//!
//! You can also the project manually, but this will lack any system integration
//! like clipboard capture:
//!
//! ```
//! cargo install --path crates/jpv --features bundle,gnome
//! ```
//!
//! <br>
//!
//! ## Configuring
//!
//! After `jpv` has been installed, you must construct the dictionary file the
//! project will use.
//!
//! ```
//! jpv build
//! ```
//!
//! After this, you can start the dictionary in the background with. This will also
//! automatically open up the interface.
//!
//! ```
//! jpv service --background
//! ```
//!
//! ![Good morning!](https://github.com/udoprog/jpv/blob/main/gfx/splash.png?raw=true)
//!
//! <br>
//!
//! ## Features
//!
//! For rust features, we have the following:
//!
//! * The `gnome` feature enabled full GNOME desktop integration, which includes the
//!   `dbus` and `ocr` features. This is also necessary to use the GNOME extension
//!   to capture the clipboard.
//! * The `dbus` feature provides the ability for the service to interact with
//!   D-Bus. Which is necessary for extensions to communicate with it and to perform
//!   D-Bus activation.
//! * The `ocr` feature provides image recognition for clipboard events where the
//!   mimetype is appropriate.
//! * The `mmap` feature (Unix only) loads the database using memory maps.
//!
//! <br>
//!
//! #### Interface
//!
//! The dictionary is primarily interacted with using the `jpv` tools. It has a
//! comprehensive help section you can get through `jpv --help`, but some of the
//! more notable features are:
//!
//! * `jpv cli <query>` can be used to perform commandline queries.
//! * `jpv send-clipboard --type text/plain hello` can be used to inject a phrase
//!   into the dictionary for analysis (requires the `dbus` feature).
//!
//! All relevant tools that interact with the background service rely on features
//! such as D-Bus activation, which will ensure that a background service is up and
//! running as needed.
//!
//! <br>
//!
//! ## Building and packing for Fedora GNOME
//!
//! To build an rpm package which is suitable for Fedora GNOME, you can do the following:
//!
//! ```sh
//! cargo build --release -p jpv --features bundle,gnome
//! cargo generate-rpm -p crates/jpv
//! ```
//!
//! The generated rpm will be located in `target/generate-rpm`.
//!
//! ```sh
//! sudo npm -i target/generate-rpm/jpv-0.0.0-1.x86_64.rpm
//! ```
//!
//! Once complete, this installs a desktop entry you can use to start the dictionary
//! in the background. Starting the application will open up the browser UI.
//!
//! Note that you still need to build the database before it can be used.
//!
//! ![Desktop entry](https://github.com/udoprog/jpv/blob/main/gfx/desktop.png?raw=true)
//!
//! <br>
//!
//! #### GNOME Extension
//!
//! Since GNOME and Wayland desktop environments in general currently do not have
//! any facilities to generically capture the clipboard we must rely on extensions.
//!
//! To enable the Japanese Dictionary extension for gnome, start the extensions
//! manager after installing the package:
//!
//! ![Gnome extension](https://github.com/udoprog/jpv/blob/main/gfx/gnome-extension.png?raw=true)
//!
//! Once enabled, clipboard capture has to be enabled in the panel item.
//!
//! <table>
//! <tr>
//! <td valign="top">
//!   <img alt="Searching for english text" src="https://github.com/udoprog/jpv/blob/main/gfx/gnome-clipboard-capture.png?raw=true" />
//!   <div style="font-size: 0.8em;">Extension button.</div>
//! </td>
//! <td valign="top">
//!   <img alt="Conjugations" src="https://github.com/udoprog/jpv/blob/main/gfx/gnome-clipboard-capture-enabled.png?raw=true" /><br>
//!   <div style="font-size: 0.8em;">Clipboard capture enabled.</div>
//! </td>
//! </tr>
//! </table>
//!
//! > **Note:** while clipboard capture is running the extension icon will be red.
//! > Only enable it while it's in use since there are currently no security
//! > mechanisms in place other than your local system. Any application could
//! > pretend to be a dictionary application and capture the clipboard.
//!
//! Clipboard capture is governed by the `capture-clipboard-enabled` setting:
//!
//! ```sh
//! > gsettings get se.tedro.japanese-dictionary.plugins capture-clipboard-enabled
//! true
//! > gsettings set se.tedro.japanese-dictionary.plugins capture-clipboard-enabled false
//! ```
//!
//! [tools/install-fedora]: https://github.com/udoprog/jpv/blob/main/tools/fedora-install

#![cfg_attr(all(not(feature = "cli"), windows), windows_subsystem = "windows")]

mod background;
mod command;
mod dbus;
mod hash;
mod log;
mod open_uri;
mod reporter;
mod system;
mod tasks;
mod web;
mod windows;

#[allow(unused)]
static VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/version.txt"));
#[allow(unused)]
static USER_AGENT: &str = include_str!(concat!(env!("OUT_DIR"), "/user_agent.txt"));

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use clap::Subcommand;
use lib::config::Config;
use lib::Dirs;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Subcommand)]
enum Command {
    /// Run as a service (default).
    Service(command::service::ServiceArgs),
    /// Perform a cli lookup.
    Cli(command::cli::CliArgs),
    /// Send clipboard to the service.
    SendClipboard(command::send_clipboard::SendClipboardArgs),
    /// Build the dictionary database. This must be performed before the cli or service can be used.
    Build(command::build::BuildArgs),
}

#[derive(Parser)]
struct Args {
    /// Specify paths to indexes to use.
    #[arg(long, value_name = "index")]
    index: Vec<PathBuf>,
    /// Command to run, by default this runs the service.
    #[command(subcommand)]
    command: Option<Command>,
}

#[tokio::main]
async fn main() -> Result<()> {
    use tracing_subscriber::Layer;

    let args = Args::try_parse()?;

    let directive = match &args.command {
        // Logging is not desired for CLI tool by default.
        Some(Command::Cli(..)) => None,
        _ => Some("jpv=info"),
    };

    let mut filter = EnvFilter::builder();

    if let Some(directive) = directive {
        filter = filter.with_default_directive(directive.parse()?);
    }

    let filter = filter.from_env_lossy();

    let system_events = system::SystemEvents::new();

    let (capturing_layer, log) = log::new(system_events.clone());

    let layer = tracing_subscriber::Registry::default();
    let layer = tracing_subscriber::fmt::layer().with_subscriber(layer);
    let layer = capturing_layer.with_subscriber(layer);
    let layer = filter.with_subscriber(layer);
    layer.try_init()?;

    let dirs = Dirs::open()?;

    let config = Config::load(&dirs).context("Loading configuration")?;

    match &args.command {
        None => {
            let service_args = Default::default();
            self::command::service::run(&args, &service_args, dirs, config, system_events, log)
                .await?;
        }
        Some(Command::Service(service_args)) => {
            self::command::service::run(&args, service_args, dirs, config, system_events, log)
                .await?;
        }
        Some(Command::Cli(cli_args)) => {
            self::command::cli::run(&args, cli_args, &dirs, config).await?;
        }
        Some(Command::SendClipboard(send_clipboard_args)) => {
            self::command::send_clipboard::run(send_clipboard_args).await?;
        }
        Some(Command::Build(build_args)) => {
            self::command::build::run(&args, build_args, &dirs, config).await?;
        }
    }

    Ok(())
}
