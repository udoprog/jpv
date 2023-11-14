use std::path::{Path, PathBuf};

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

    /// Get dictionary path.
    pub(crate) fn dictionary(&self) -> PathBuf {
        self.data_dir("dict.bin")
    }

    /// Construct a path inside of the data directory.
    fn data_dir<P>(&self, path: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        self.project_dirs.data_dir().join(path)
    }

    /// Construct a path inside of the cache directory.
    pub(crate) fn cache_dir<P>(&self, path: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        self.project_dirs.cache_dir().join(path)
    }
}
