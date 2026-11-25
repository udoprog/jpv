//! `jpv update`: replaces the running binary with the prebuilt one for this
//! platform from the latest release, the same archive `install.sh` installs.
//!
//! Release archives are named `jpv-<version>-<arch>-<os>.tar.gz`, or `.zip` on
//! Windows, and hold the binary at their root.

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub(crate) struct UpdateArgs {
    /// Install the latest release even if it is not newer than the running
    /// version, or the running version cannot be compared, as with builds
    /// from source.
    #[arg(long)]
    force: bool,
}

#[cfg(not(feature = "reqwest"))]
pub(crate) async fn run(_: &UpdateArgs) -> Result<()> {
    anyhow::bail!("jpv was built without the `reqwest` feature and cannot download updates")
}

#[cfg(feature = "reqwest")]
pub(crate) async fn run(args: &UpdateArgs) -> Result<()> {
    imp::run(args).await
}

/// Whether `latest` is newer than `current`, comparing the leading numeric
/// segments split on `.` and `-`, so both `1.2.3` and dated `2026-05-04`
/// releases order as expected. `None` if either is not a version.
#[cfg_attr(not(feature = "reqwest"), allow(unused))]
fn is_newer(latest: &str, current: &str) -> Option<bool> {
    Some(parse_version(latest)? > parse_version(current)?)
}

fn parse_version(version: &str) -> Option<Vec<u64>> {
    let mut parts = version
        .trim_start_matches('v')
        .split(['.', '-'])
        .map_while(|p| p.parse::<u64>().ok())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return None;
    }

    while parts.len() > 1 && parts.last() == Some(&0) {
        parts.pop();
    }

    Some(parts)
}

/// The name of the release archive for `arch`-`os` ends with this.
#[cfg_attr(not(feature = "reqwest"), allow(unused))]
fn asset_suffix(arch: &str, os: &str) -> String {
    let ext = if os == "windows" { "zip" } else { "tar.gz" };
    format!("-{arch}-{os}.{ext}")
}

#[cfg(feature = "reqwest")]
mod imp {
    use std::env::consts::{ARCH, EXE_SUFFIX, OS};
    use std::fs::{self, File};
    use std::io;
    use std::path::{Path, PathBuf};

    use anyhow::{bail, Context, Result};
    use serde::Deserialize;

    use super::UpdateArgs;

    const LATEST: &str = "https://api.github.com/repos/udoprog/jpv/releases/latest";

    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
        assets: Vec<Asset>,
    }

    #[derive(Deserialize)]
    struct Asset {
        name: String,
        browser_download_url: String,
    }

    pub(super) async fn run(args: &UpdateArgs) -> Result<()> {
        let current = crate::VERSION;

        let exe = std::env::current_exe()
            .and_then(fs::canonicalize)
            .context("Locating the current executable")?;

        ensure_self_managed(&exe)?;

        let client = reqwest::ClientBuilder::new()
            .user_agent(crate::USER_AGENT)
            .build()?;

        let release = client
            .get(LATEST)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .context("Looking up the latest release")?
            .bytes()
            .await
            .context("Looking up the latest release")?;

        let release: Release =
            serde_json::from_slice(&release).context("Reading the latest release")?;

        let latest = release.tag_name.trim_start_matches('v');

        if !args.force {
            match super::is_newer(latest, current) {
                Some(true) => {}
                Some(false) => {
                    println!("jpv {current} is up to date");
                    return Ok(());
                }
                None => bail!(
                    "Cannot tell whether {latest} is newer than this build ({current}); pass --force to install it anyway"
                ),
            }
        }

        let Some(asset) = select_asset(&release.assets, ARCH, OS) else {
            bail!("The latest release {latest} has no prebuilt jpv for {ARCH}-{OS}");
        };

        let dir = exe
            .parent()
            .context("Current executable has no directory")?;

        // Next to the executable, so the new binary is renamed into place on
        // the same filesystem.
        let work = dir.join(format!(".jpv-update{}", std::process::id()));
        _ = fs::remove_dir_all(&work);
        fs::create_dir_all(&work).with_context(|| work.display().to_string())?;

        println!("Downloading {}", asset.name);

        let result = async {
            let bytes = client
                .get(&asset.browser_download_url)
                .send()
                .await
                .and_then(|r| r.error_for_status())
                .with_context(|| asset.browser_download_url.clone())?
                .bytes()
                .await
                .with_context(|| asset.browser_download_url.clone())?;

            let archive = work.join(&asset.name);
            fs::write(&archive, &bytes).with_context(|| archive.display().to_string())?;
            let new = work.join(format!("jpv{EXE_SUFFIX}"));
            extract_binary(&archive, &new)?;
            replace(&new, &exe)
        }
        .await;

        _ = fs::remove_dir_all(&work);
        result?;

        println!("jpv updated from {current} to {latest}");
        println!("Restart the jpv service if it is running to use the new version");
        Ok(())
    }

    /// Refuses to replace a binary a package manager owns, so it is updated
    /// the same way it was installed.
    fn ensure_self_managed(exe: &Path) -> Result<()> {
        if cfg!(unix) && exe.starts_with("/usr") {
            bail!(
                "{} is managed by the system package manager; update jpv through it instead",
                exe.display()
            );
        }

        if cfg!(windows) {
            for var in ["ProgramFiles", "ProgramFiles(x86)"] {
                if let Some(dir) = std::env::var_os(var) {
                    if exe.starts_with(&dir) {
                        bail!(
                            "{} was installed by the MSI; install the new MSI from https://github.com/udoprog/jpv/releases instead",
                            exe.display()
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn select_asset<'a>(assets: &'a [Asset], arch: &str, os: &str) -> Option<&'a Asset> {
        let suffix = super::asset_suffix(arch, os);

        assets
            .iter()
            .find(|a| a.name.starts_with("jpv-") && a.name.ends_with(&suffix))
    }

    fn extract_binary(archive: &Path, to: &Path) -> Result<()> {
        let name = format!("jpv{EXE_SUFFIX}");
        let context = || archive.display().to_string();
        let file = File::open(archive).with_context(context)?;
        let out = || File::create(to).with_context(|| to.display().to_string());

        #[cfg(windows)]
        {
            let mut zip = zip::ZipArchive::new(io::BufReader::new(file)).with_context(context)?;
            let mut entry = zip
                .by_name(&name)
                .with_context(|| format!("{}: {name}", archive.display()))?;
            io::copy(&mut entry, &mut out()?).with_context(context)?;
            Ok(())
        }

        #[cfg(not(windows))]
        {
            let gz = flate2::read::GzDecoder::new(io::BufReader::new(file));
            let mut tar = tar::Archive::new(gz);

            for entry in tar.entries().with_context(context)? {
                let mut entry = entry.with_context(context)?;
                let path = entry.path().with_context(context)?;

                if path.components().eq(Path::new(&name).components()) {
                    io::copy(&mut entry, &mut out()?).with_context(context)?;
                    return Ok(());
                }
            }

            bail!("{}: No {name} in the archive", archive.display())
        }
    }

    /// Renames `new` over `exe`. On Windows the running executable is first
    /// moved aside to `jpv.old.exe`, and restored if the rename fails.
    fn replace(new: &Path, exe: &Path) -> Result<()> {
        let context = || format!("Replacing {}", exe.display());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(new, fs::Permissions::from_mode(0o755))
                .with_context(|| new.display().to_string())?;
        }

        if cfg!(windows) {
            let old: PathBuf = exe.with_file_name("jpv.old.exe");
            _ = fs::remove_file(&old);
            fs::rename(exe, &old).with_context(context)?;

            if let Err(e) = fs::rename(new, exe) {
                _ = fs::rename(&old, exe);
                return Err(e).with_context(context);
            }

            return Ok(());
        }

        fs::rename(new, exe).with_context(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_numerically() {
        let newer = |a, b| is_newer(a, b).unwrap();
        assert!(newer("0.1.1", "0.1.0"));
        assert!(newer("0.10.0", "0.9.9"));
        assert!(newer("1.0", "0.9.9"));
        assert!(newer("v0.2.0", "0.1.0"));
        assert!(!newer("0.1.0", "0.1.0"));
        assert!(!newer("0.1", "0.1.0"));
        assert!(!newer("0.0.9", "0.1.0"));
        assert!(newer("2026-05-04", "2026-05-03"));
        assert!(newer("2026-05-04", "2026-04-30"));
        assert!(newer("2026-05-04-1", "2026-05-04"));
        assert!(is_newer("0.1.0", "git-abc1234").is_none());
        assert!(is_newer("nightly", "0.1.0").is_none());
    }

    #[test]
    fn names_platform_archives() {
        assert_eq!(asset_suffix("x86_64", "linux"), "-x86_64-linux.tar.gz");
        assert_eq!(asset_suffix("aarch64", "macos"), "-aarch64-macos.tar.gz");
        assert_eq!(asset_suffix("x86_64", "windows"), "-x86_64-windows.zip");
    }
}
