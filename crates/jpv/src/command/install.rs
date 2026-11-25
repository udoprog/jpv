//! Per-user desktop integration: `jpv install` and `jpv uninstall`.
//!
//! Every file is embedded in the binary and written under `$XDG_DATA_HOME`
//! (by default `~/.local/share`). Integration is grouped into components so
//! that more desktops can be added as one more entry in [`components`].

use std::ffi::{OsStr, OsString};
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
    /// Installs and uninstalls the component through an external tool,
    /// falling back to writing [`Component::files`] directly.
    hook: Option<&'static Hook>,
}

/// How install and uninstall are carried out.
struct Env<'a> {
    /// The data home everything is installed under.
    root: &'a Path,
    /// The running binary, for `Exec=` lines.
    exe: &'a str,
    /// Whether external tools such as kpackagetool6 may be used.
    tools: bool,
}

/// An external tool which installs or uninstalls a component.
///
/// Each returns `Ok(false)` when the tool is not available, in which case the
/// files are written or removed directly.
struct Hook {
    install: fn(&Env<'_>, &Component) -> Result<bool>,
    uninstall: fn(&Env<'_>, &Component) -> Result<bool>,
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
    hook: None,
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
    hook: None,
};

/// The plasmoid id, which is also its directory name.
const PLASMOID_ID: &str = "se.tedro.JapaneseDictionary";
/// Where Plasma looks for per-user plasmoids, relative to the data home.
const PLASMOID_DIR: &str = "plasma/plasmoids/se.tedro.JapaneseDictionary";

const KDE: Component = Component {
    name: "KDE Plasma widget",
    files: &[
        File {
            path: "plasma/plasmoids/se.tedro.JapaneseDictionary/metadata.json",
            contents: Contents::Static(asset!("kde/se.tedro.JapaneseDictionary/metadata.json")),
        },
        File {
            path: "plasma/plasmoids/se.tedro.JapaneseDictionary/contents/ui/main.qml",
            contents: Contents::Static(asset!(
                "kde/se.tedro.JapaneseDictionary/contents/ui/main.qml"
            )),
        },
    ],
    dirs: &[
        "plasma/plasmoids/se.tedro.JapaneseDictionary/contents/ui",
        "plasma/plasmoids/se.tedro.JapaneseDictionary/contents",
        "plasma/plasmoids/se.tedro.JapaneseDictionary",
    ],
    hint: Some(
        "Add the Japanese Dictionary widget to a Plasma panel with \"Add or Manage Widgets…\". Start the service with clipboard capture (the `wayland` feature) for the \"Capture clipboard\" switch to have an effect.",
    ),
    hook: Some(&Hook {
        install: kpackage_install,
        uninstall: kpackage_uninstall,
    }),
};

fn components() -> [&'static Component; 3] {
    [&DESKTOP, &GNOME, &KDE]
}

const KPACKAGETOOL: &str = "kpackagetool6";

/// Run kpackagetool6 against the data home, returning `Ok(false)` if it is
/// not installed.
fn kpackagetool(env: &Env<'_>, args: &[&OsStr]) -> Result<bool> {
    if !env.tools {
        return Ok(false);
    }

    let output = match std::process::Command::new(KPACKAGETOOL)
        .args(["--type", "Plasma/Applet"])
        .args(args)
        // kpackagetool6 installs into the first XDG data directory, so point
        // it at the same one everything else goes to.
        .env("XDG_DATA_HOME", env.root)
        .stdin(std::process::Stdio::null())
        .output()
    {
        Ok(output) => output,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e).with_context(|| format!("Running {KPACKAGETOOL}")),
    };

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "{KPACKAGETOOL} {:?} exited with {}: {}",
            args,
            output.status,
            format!("{stdout}{stderr}").trim()
        );
    }

    Ok(true)
}

/// Write the package to a staging directory and install or upgrade it with
/// kpackagetool6.
fn kpackage_install(env: &Env<'_>, component: &Component) -> Result<bool> {
    let dir = env.root.join(PLASMOID_DIR);

    // An empty directory left behind keeps kpackagetool6 from installing, so
    // remove it. Fails, and is ignored, unless it is empty.
    let _ = std::fs::remove_dir(&dir);

    let mode = if dir.join("metadata.json").is_file() {
        "--upgrade"
    } else if dir.exists() {
        // kpackagetool6 neither installs over nor upgrades a directory which
        // is not a package, so leave it to writing the files directly.
        return Ok(false);
    } else {
        "--install"
    };

    let staging = tempfile::tempdir().context("Creating a staging directory")?;
    let package = staging.path().join(PLASMOID_ID);
    write_files(&package, component, env.exe, |path| {
        path.strip_prefix(PLASMOID_DIR).ok()
    })?;

    kpackagetool(env, &[OsStr::new(mode), package.as_os_str()])
}

/// Remove the package with kpackagetool6, but only when it holds nothing
/// but the files install writes, since kpackagetool6 removes the whole
/// directory.
fn kpackage_uninstall(env: &Env<'_>, component: &Component) -> Result<bool> {
    let dir = env.root.join(PLASMOID_DIR);

    if !dir.join("metadata.json").is_file() || !only_contains(env.root, &dir, component)? {
        return Ok(false);
    }

    kpackagetool(env, &[OsStr::new("--remove"), OsStr::new(PLASMOID_ID)])
}

/// Test that everything under `dir` is a file or directory of `component`.
fn only_contains(root: &Path, dir: &Path, component: &Component) -> Result<bool> {
    let entries = std::fs::read_dir(dir).with_context(|| format!("Reading {}", dir.display()))?;

    for entry in entries {
        let path = entry
            .with_context(|| format!("Reading {}", dir.display()))?
            .path();

        let Ok(relative) = path.strip_prefix(root) else {
            return Ok(false);
        };

        if path.is_dir() && !path.is_symlink() {
            if !component.dirs.iter().any(|d| Path::new(d) == relative) {
                return Ok(false);
            }

            if !only_contains(root, &path, component)? {
                return Ok(false);
            }
        } else if !component
            .files
            .iter()
            .any(|f| Path::new(f.path) == relative)
        {
            return Ok(false);
        }
    }

    Ok(true)
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

/// Write the files of `component` under `root`, at the path `map` gives
/// each, skipping files it gives none for.
fn write_files<'a>(
    root: &Path,
    component: &'a Component,
    exe: &str,
    map: impl Fn(&'a Path) -> Option<&'a Path>,
) -> Result<Vec<PathBuf>> {
    let mut written = Vec::new();

    for file in component.files {
        let Some(relative) = map(Path::new(file.path)) else {
            continue;
        };

        let path = root.join(relative);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Creating {}", parent.display()))?;
        }

        std::fs::write(&path, render(&file.contents, exe))
            .with_context(|| format!("Writing {}", path.display()))?;

        written.push(path);
    }

    Ok(written)
}

fn install_into(env: &Env<'_>) -> Result<Vec<(&'static str, Vec<PathBuf>)>> {
    let mut report = Vec::new();

    for component in components() {
        let hooked = match component.hook {
            Some(hook) => (hook.install)(env, component)?,
            None => false,
        };

        let written = if hooked {
            component
                .files
                .iter()
                .map(|f| env.root.join(f.path))
                .collect()
        } else {
            write_files(env.root, component, env.exe, Some)?
        };

        report.push((component.name, written));
    }

    Ok(report)
}

fn uninstall_from(env: &Env<'_>) -> Result<Vec<PathBuf>> {
    let mut removed = Vec::new();

    for component in components() {
        if let Some(hook) = component.hook {
            let existing = component
                .files
                .iter()
                .map(|f| env.root.join(f.path))
                .filter(|p| p.is_file())
                .collect::<Vec<_>>();

            if (hook.uninstall)(env, component)? {
                removed.extend(existing);
                continue;
            }
        }

        for file in component.files {
            let path = env.root.join(file.path);

            match std::fs::remove_file(&path) {
                Ok(()) => removed.push(path),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).with_context(|| format!("Removing {}", path.display())),
            }
        }

        for dir in component.dirs {
            let path = env.root.join(dir);

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

    let env = Env {
        root: &root,
        exe,
        tools: true,
    };

    let report = install_into(&env)?;
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
    let removed = uninstall_from(&Env {
        root: &root,
        exe: TEMPLATE_EXE,
        tools: true,
    })?;
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

    fn env(root: &Path, tools: bool) -> Env<'_> {
        Env {
            root,
            exe: "/opt/jpv",
            tools,
        }
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
            "/d/plasma/plasmoids/se.tedro.JapaneseDictionary/metadata.json",
            "/d/plasma/plasmoids/se.tedro.JapaneseDictionary/contents/ui/main.qml",
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

        install_into(&env(root, false)).unwrap();
        let installed = installed_files(root).collect::<BTreeSet<_>>();
        assert!(installed.iter().all(|p| p.is_file()));

        let removed = uninstall_from(&env(root, false)).unwrap();
        assert_eq!(removed.into_iter().collect::<BTreeSet<_>>(), installed);

        let after = tree(root);
        assert!(after.is_superset(&before));
        assert!(after.iter().all(|p| p.is_dir() || before.contains(p)));
        assert!(!root.join("gnome-shell/extensions/jpv@tedro.se").exists());
        assert!(!root.join(PLASMOID_DIR).exists());

        // Idempotent.
        assert!(uninstall_from(&env(root, false)).unwrap().is_empty());
        assert_eq!(tree(root), after);
    }

    #[test]
    fn uninstall_keeps_user_files_in_extension_dir() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        install_into(&env(root, false)).unwrap();
        let extra = root.join("gnome-shell/extensions/jpv@tedro.se/notes.txt");
        std::fs::write(&extra, "mine").unwrap();

        uninstall_from(&env(root, false)).unwrap();
        assert!(extra.is_file());
    }

    #[test]
    fn plasmoid_package_matches_files() {
        let package =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../kde/se.tedro.JapaneseDictionary");
        let mut on_disk = BTreeSet::new();
        walk(&package, &mut on_disk);

        let on_disk = on_disk
            .into_iter()
            .filter(|p| p.is_file())
            .map(|p| Path::new(PLASMOID_DIR).join(p.strip_prefix(&package).unwrap()))
            .collect::<BTreeSet<_>>();

        let embedded = KDE
            .files
            .iter()
            .map(|f| PathBuf::from(f.path))
            .collect::<BTreeSet<_>>();

        assert_eq!(on_disk, embedded);
    }

    /// The kpackagetool6 path, which only runs where it is installed. It only
    /// ever writes into the temporary data home.
    #[test]
    fn kpackagetool_install_and_uninstall() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let env = env(root, true);

        if !kpackagetool(&env, &[OsStr::new("--list")]).unwrap_or(false) {
            return;
        }

        let before = tree(root);

        install_into(&env).unwrap();
        let installed = installed_files(root).collect::<BTreeSet<_>>();
        assert!(installed.iter().all(|p| p.is_file()));

        // Upgrading over an existing install.
        install_into(&env).unwrap();

        // A user file in the package directory keeps kpackagetool6 from
        // removing it.
        let extra = root.join(PLASMOID_DIR).join("notes.txt");
        std::fs::write(&extra, "mine").unwrap();
        uninstall_from(&env).unwrap();
        assert!(extra.is_file());
        std::fs::remove_file(&extra).unwrap();

        install_into(&env).unwrap();
        let removed = uninstall_from(&env).unwrap();
        assert_eq!(removed.into_iter().collect::<BTreeSet<_>>(), installed);
        assert!(!root.join(PLASMOID_DIR).exists());

        let after = tree(root);
        assert!(after.iter().all(|p| p.is_dir() || before.contains(p)));
        assert!(uninstall_from(&env).unwrap().is_empty());
    }
}
