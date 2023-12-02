use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use directories::ProjectDirs;

pub(crate) struct Dirs {
    project_dirs: ProjectDirs,
}

impl Dirs {
    /// Open directories for this project.
    pub(crate) fn open() -> Result<Dirs> {
        Ok(Dirs {
            project_dirs: directories::ProjectDirs::from("se", "tedro", "jpv")
                .context("Could not figure out base directories")?,
        })
    }

    /// Get the path of the configuration file.
    pub(crate) fn config_path(&self) -> PathBuf {
        self.project_dirs.config_dir().join("config.toml")
    }

    /// The path to an individual index.
    pub(crate) fn index_path(&self, name: &str) -> PathBuf {
        self.project_dirs.data_dir().join(format!("{name}.index"))
    }

    /// Get dictionary path.
    pub(crate) fn indexes(&self) -> Result<Vec<PathBuf>> {
        let mut indexes = Vec::new();

        let d = fs::read_dir(self.project_dirs.data_dir())?;

        for e in d {
            let e = e?;
            let path = e.path();

            if path.extension() != Some("index".as_ref()) {
                continue;
            }

            if path.is_file() {
                indexes.push(path);
            }
        }

        Ok(indexes)
    }

    /// Construct a path inside of the cache directory.
    pub(crate) fn cache_dir<P>(&self, path: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        self.project_dirs.cache_dir().join(path)
    }
}
