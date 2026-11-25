//! Per-user desktop integration: `jpv install` and `jpv uninstall`.
//!
//! Every file is embedded in the binary and written under `$XDG_DATA_HOME`
//! (by default `~/.local/share`). Integration is grouped into components so
//! that more desktops can be added as one more entry in [`components`].

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;

#[derive(Parser)]
pub(crate) struct InstallArgs {}

#[derive(Parser)]
pub(crate) struct UninstallArgs {}

macro_rules! asset {
    ($path:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../", $path))
    };
}

const DESKTOP_ENTRY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../desktop/se.tedro.JapaneseDictionary.desktop"
));
const DBUS_SERVICE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../desktop/se.tedro.JapaneseDictionary.service"
));
const ICON: &[u8] = asset!("desktop/se.tedro.JapaneseDictionary.png");

/// The binary path the embedded templates refer to.
const TEMPLATE_EXE: &str = "/usr/bin/jpv";

/// The contents of one installed file.
enum Contents {
    /// Written as is.
    Static(&'static [u8]),
    /// Text where [`TEMPLATE_EXE`] is replaced with the running binary.
    Exec(&'static str),
}

/// One installed file, relative to the data home.
struct File {
    path: &'static str,
    contents: Contents,
}

/// A set of files which make up one piece of desktop integration.
struct Component {
    name: &'static str,
    files: &'static [File],
    /// Directories relative to the data home which belong to this component
    /// alone, removed on uninstall once empty. Listed innermost first.
    dirs: &'static [&'static str],
    /// What to tell the user after installing.
    hint: Option<&'static str>,
}

const DESKTOP: Component = Component {
    name: "desktop entry",
    files: &[
        File {
            path: "applications/se.tedro.JapaneseDictionary.desktop",
            contents: Contents::Exec(DESKTOP_ENTRY),
        },
        File {
            path: "icons/hicolor/256x256/apps/se.tedro.JapaneseDictionary.png",
            contents: Contents::Static(ICON),
        },
        File {
            path: "dbus-1/services/se.tedro.JapaneseDictionary.service",
            contents: Contents::Exec(DBUS_SERVICE),
        },
    ],
    dirs: &[],
    hint: None,
};

const GNOME: Component = Component {
    name: "GNOME Shell extension",
    files: &[
        File {
            path: "gnome-shell/extensions/jpv@tedro.se/extension.js",
            contents: Contents::Static(asset!("gnome/jpv@tedro.se/extension.js")),
        },
        File {
            path: "gnome-shell/extensions/jpv@tedro.se/prefs.js",
            contents: Contents::Static(asset!("gnome/jpv@tedro.se/prefs.js")),
        },
        File {
            path: "gnome-shell/extensions/jpv@tedro.se/metadata.json",
            contents: Contents::Static(asset!("gnome/jpv@tedro.se/metadata.json")),
        },
        File {
            path: "gnome-shell/extensions/jpv@tedro.se/stylesheet.css",
            contents: Contents::Static(asset!("gnome/jpv@tedro.se/stylesheet.css")),
        },
        File {
            path: "gnome-shell/extensions/jpv@tedro.se/schemas/se.tedro.japanese-dictionary.plugins.gschema.xml",
            contents: Contents::Static(asset!(
                "gnome/jpv@tedro.se/schemas/se.tedro.japanese-dictionary.plugins.gschema.xml"
            )),
        },
        File {
            path: "gnome-shell/extensions/jpv@tedro.se/schemas/gschemas.compiled",
            contents: Contents::Static(asset!("gnome/jpv@tedro.se/schemas/gschemas.compiled")),
        },
    ],
    dirs: &[
        "gnome-shell/extensions/jpv@tedro.se/schemas",
        "gnome-shell/extensions/jpv@tedro.se",
    ],
    hint: Some(
        "Enable it with `gnome-extensions enable jpv@tedro.se`. On Wayland, log out and back in first so GNOME Shell sees it.",
    ),
};

fn components() -> [&'static Component; 2] {
    [&DESKTOP, &GNOME]
}

/// The per-user data directory, following the XDG base directory spec:
/// `$XDG_DATA_HOME` when it is an absolute path, else `$HOME/.local/share`.
fn data_home(xdg_data_home: Option<OsString>, home: Option<OsString>) -> Result<PathBuf> {
    if let Some(dir) = xdg_data_home.map(PathBuf::from) {
        if dir.is_absolute() {
            return Ok(dir);
        }
    }

    let Some(home) = home.map(PathBuf::from).filter(|h| h.is_absolute()) else {
        bail!("Neither XDG_DATA_HOME nor HOME is set to an absolute path");
    };

    Ok(home.join(".local").join("share"))
}

fn env_data_home() -> Result<PathBuf> {
    data_home(std::env::var_os("XDG_DATA_HOME"), std::env::var_os("HOME"))
}

/// Quote a path for an `Exec=` key, as both the desktop entry spec and D-Bus
/// service files accept.
fn quote_exec(exe: &str) -> String {
    let plain = exe
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '-' | '_' | '+'));

    if plain {
        return exe.to_owned();
    }

    let mut out = String::with_capacity(exe.len() + 2);
    out.push('"');

    for c in exe.chars() {
        if matches!(c, '"' | '`' | '$' | '\\') {
            out.push('\\');
        }

        out.push(c);
    }

    out.push('"');
    out
}

fn render(contents: &Contents, exe: &str) -> Vec<u8> {
    match contents {
        Contents::Static(bytes) => bytes.to_vec(),
        Contents::Exec(text) => text.replace(TEMPLATE_EXE, &quote_exec(exe)).into_bytes(),
    }
}

fn install_into(root: &Path, exe: &str) -> Result<Vec<(&'static str, Vec<PathBuf>)>> {
    let mut report = Vec::new();

    for component in components() {
        let mut written = Vec::new();

        for file in component.files {
            let path = root.join(file.path);

            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Creating {}", parent.display()))?;
            }

            std::fs::write(&path, render(&file.contents, exe))
                .with_context(|| format!("Writing {}", path.display()))?;

            written.push(path);
        }

        report.push((component.name, written));
    }

    Ok(report)
}

fn uninstall_from(root: &Path) -> Result<Vec<PathBuf>> {
    let mut removed = Vec::new();

    for component in components() {
        for file in component.files {
            let path = root.join(file.path);

            match std::fs::remove_file(&path) {
                Ok(()) => removed.push(path),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).with_context(|| format!("Removing {}", path.display())),
            }
        }

        for dir in component.dirs {
            let path = root.join(dir);

            // Only removes empty directories, so anything the user put there
            // is left alone.
            match std::fs::remove_dir(&path) {
                Ok(()) => {}
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
                    ) => {}
                Err(e) => return Err(e).with_context(|| format!("Removing {}", path.display())),
            }
        }
    }

    Ok(removed)
}

/// Refresh desktop caches, skipping tools which are not installed.
fn refresh_caches(root: &Path) {
    let applications = root.join("applications");
    let hicolor = root.join("icons").join("hicolor");

    let commands: [(&str, Vec<OsString>); 2] = [
        ("update-desktop-database", vec![applications.into()]),
        (
            "gtk-update-icon-cache",
            vec!["-f".into(), "-t".into(), "-q".into(), hicolor.into()],
        ),
    ];

    for (program, args) in commands {
        match std::process::Command::new(program)
            .args(&args)
            .stdout(std::process::Stdio::null())
            .status()
        {
            Ok(status) if !status.success() => {
                tracing::warn!("{program} exited with {status}");
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => tracing::warn!("Running {program}: {e}"),
        }
    }
}

fn ensure_supported() -> Result<()> {
    if !cfg!(target_os = "linux") {
        bail!("Desktop integration is only supported on Linux");
    }

    Ok(())
}

pub(crate) fn install(_: &InstallArgs) -> Result<()> {
    ensure_supported()?;

    let root = env_data_home()?;
    let exe = std::env::current_exe().context("Finding the running binary")?;
    let Some(exe) = exe.to_str() else {
        bail!("The path to jpv is not valid UTF-8: {}", exe.display());
    };

    let report = install_into(&root, exe)?;
    refresh_caches(&root);

    for (name, files) in report {
        println!("Installed {name}:");

        for file in files {
            println!("  {}", file.display());
        }
    }

    for component in components() {
        if let Some(hint) = component.hint {
            println!();
            println!("{hint}");
        }
    }

    Ok(())
}

pub(crate) fn uninstall(_: &UninstallArgs) -> Result<()> {
    ensure_supported()?;

    let root = env_data_home()?;
    let removed = uninstall_from(&root)?;
    refresh_caches(&root);

    if removed.is_empty() {
        println!("Nothing to uninstall under {}", root.display());
    } else {
        println!("Removed:");

        for path in removed {
            println!("  {}", path.display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::Path;

    use super::*;

    /// Every path install writes, relative to `root`.
    fn installed_files(root: &Path) -> impl Iterator<Item = PathBuf> + '_ {
        components()
            .into_iter()
            .flat_map(|c| c.files.iter())
            .map(move |f| root.join(f.path))
    }

    fn walk(root: &Path, out: &mut BTreeSet<PathBuf>) {
        for entry in std::fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            out.insert(path.clone());

            if path.is_dir() {
                walk(&path, out);
            }
        }
    }

    fn tree(root: &Path) -> BTreeSet<PathBuf> {
        let mut out = BTreeSet::new();
        walk(root, &mut out);
        out
    }

    #[test]
    fn data_home_layout() {
        assert_eq!(
            data_home(Some("/x/data".into()), Some("/home/u".into())).unwrap(),
            Path::new("/x/data")
        );
        assert_eq!(
            data_home(Some("relative".into()), Some("/home/u".into())).unwrap(),
            Path::new("/home/u/.local/share")
        );
        assert_eq!(
            data_home(None, Some("/home/u".into())).unwrap(),
            Path::new("/home/u/.local/share")
        );
        assert!(data_home(None, None).is_err());
    }

    #[test]
    fn paths() {
        let files = installed_files(Path::new("/d")).collect::<BTreeSet<_>>();

        for expected in [
            "/d/applications/se.tedro.JapaneseDictionary.desktop",
            "/d/icons/hicolor/256x256/apps/se.tedro.JapaneseDictionary.png",
            "/d/dbus-1/services/se.tedro.JapaneseDictionary.service",
            "/d/gnome-shell/extensions/jpv@tedro.se/metadata.json",
            "/d/gnome-shell/extensions/jpv@tedro.se/schemas/gschemas.compiled",
        ] {
            assert!(files.contains(Path::new(expected)), "{expected}");
        }

        for component in components() {
            for dir in component.dirs {
                assert!(component.files.iter().any(|f| f.path.starts_with(dir)));
            }
        }
    }

    #[test]
    fn exec_points_at_binary() {
        assert!(DESKTOP_ENTRY.contains(TEMPLATE_EXE));
        assert!(DBUS_SERVICE.contains(TEMPLATE_EXE));

        let text = String::from_utf8(render(&Contents::Exec(DESKTOP_ENTRY), "/opt/jpv")).unwrap();
        assert!(text.contains("Exec=/opt/jpv service --background"));

        let text = String::from_utf8(render(&Contents::Exec(DBUS_SERVICE), "/a b/jpv")).unwrap();
        assert!(text.contains("Exec=\"/a b/jpv\" service --no-open"));

        assert_eq!(quote_exec("/x/$y\"z"), "\"/x/\\$y\\\"z\"");
    }

    #[test]
    fn uninstall_removes_exactly_install() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let keep = root.join("applications/other.desktop");
        std::fs::create_dir_all(keep.parent().unwrap()).unwrap();
        std::fs::write(&keep, "x").unwrap();
        let before = tree(root);

        install_into(root, "/opt/jpv").unwrap();
        let installed = installed_files(root).collect::<BTreeSet<_>>();
        assert!(installed.iter().all(|p| p.is_file()));

        let removed = uninstall_from(root).unwrap();
        assert_eq!(removed.into_iter().collect::<BTreeSet<_>>(), installed);

        let after = tree(root);
        assert!(after.is_superset(&before));
        assert!(after.iter().all(|p| p.is_dir() || before.contains(p)));
        assert!(!root.join("gnome-shell/extensions/jpv@tedro.se").exists());

        // Idempotent.
        assert!(uninstall_from(root).unwrap().is_empty());
        assert_eq!(tree(root), after);
    }

    #[test]
    fn uninstall_keeps_user_files_in_extension_dir() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        install_into(root, "/opt/jpv").unwrap();
        let extra = root.join("gnome-shell/extensions/jpv@tedro.se/notes.txt");
        std::fs::write(&extra, "mine").unwrap();

        uninstall_from(root).unwrap();
        assert!(extra.is_file());
    }
}
