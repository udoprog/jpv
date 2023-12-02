use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use lib::Dirs;

const JMDICT_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/JMdict_e_examp.gz";
const KANJIDIC2_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/kanjidic2.xml.gz";
const JMNEDICT_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/JMnedict.xml.gz";

/// Download override paths.
#[derive(Default)]
pub struct DownloadOverrides<'a> {
    pub jmdict_path: Option<&'a Path>,
    pub kanjidic2_path: Option<&'a Path>,
    pub jmnedict_path: Option<&'a Path>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IndexKind {
    Jmdict,
    Kanjidic2,
    Jmnedict,
}

/// An index.
#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    kind: IndexKind,
}

/// A configuration used for the application.
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Enabled indexes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    indexes: Vec<Index>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            indexes: vec![
                Index {
                    kind: IndexKind::Jmdict,
                },
                Index {
                    kind: IndexKind::Kanjidic2,
                },
                Index {
                    kind: IndexKind::Jmnedict,
                },
            ],
        }
    }
}

impl Config {
    pub fn load(dirs: &Dirs) -> Result<Self> {
        let config_path = dirs.config_path();

        let config = if config_path.exists() {
            let data = std::fs::read_to_string(&config_path)?;
            toml::from_str(&data)?
        } else {
            Self::default()
        };

        Ok(config)
    }

    /// Convert configuration into indexes that should be downloaded and built.
    pub fn to_download(&self, dirs: &Dirs, overrides: DownloadOverrides<'_>) -> Vec<ToDownload> {
        let mut downloads = Vec::new();

        for index in &self.indexes {
            let download = match index.kind {
                IndexKind::Jmdict => ToDownload {
                    name: "jmdict".into(),
                    url: JMDICT_URL.into(),
                    url_name: "JMdict_e_examp.gz".into(),
                    index_path: dirs.index_path("jmdict").into(),
                    path: overrides.jmdict_path.map(Into::into),
                    kind: index.kind,
                },
                IndexKind::Kanjidic2 => ToDownload {
                    name: "kanjidic2".into(),
                    url: KANJIDIC2_URL.into(),
                    url_name: "kanjidic2.xml.gz".into(),
                    index_path: dirs.index_path("kanjidic2").into(),
                    path: overrides.kanjidic2_path.map(Into::into),
                    kind: index.kind,
                },
                IndexKind::Jmnedict => ToDownload {
                    name: "jmnedict".into(),
                    url: JMNEDICT_URL.into(),
                    url_name: "jmnedict.xml.gz".into(),
                    index_path: dirs.index_path("jmnedict").into(),
                    path: overrides.jmnedict_path.map(Into::into),
                    kind: index.kind,
                },
            };

            downloads.push(download);
        }

        downloads
    }
}

/// Path and url to download.
pub struct ToDownload {
    pub name: String,
    pub url: String,
    pub url_name: String,
    pub index_path: Box<Path>,
    pub path: Option<Box<Path>>,
    pub kind: IndexKind,
}
