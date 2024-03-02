use std::collections::BTreeMap;
use std::str::FromStr;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Dirs;

const JMDICT_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/JMdict_e_examp.gz";
const JMDICT_HELP: &str = "https://www.edrdg.org/wiki/index.php/JMdict-EDICT_Dictionary_Project";
const JMDICT_DESCRIPTION: &str = "JMDict (with examples)";

const JMNEDICT_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/JMnedict.xml.gz";
const JMNEDICT_HELP: &str =
    "https://www.edrdg.org/wiki/index.php/Main_Page#The_ENAMDICT/JMnedict_Project";
const JMNEDICT_DESCRIPTION: &str = "Names from JMnedict";

const KANJIDIC2_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/kanjidic2.xml.gz";
const KANJIDIC2_HELP: &str = "https://www.edrdg.org/wiki/index.php/KANJIDIC_Project";
const KANJIDIC2_DESCRIPTION: &str = "Kanji from Kanjidic2";

const KRADFILE_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/kradfile.gz";
const KRADFILE_HELP: &str = "https://www.edrdg.org/krad/kradinf.html";
const KRADFILE_DESCRIPTION: &str = "Radicals from KRADFILE";

#[derive(Debug, Error)]
#[error("Invalid index format")]
#[non_exhaustive]
pub struct IndexFormatError;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum IndexFormat {
    #[default]
    Jmdict,
    Jmnedict,
    Kanjidic2,
    Kradfile,
}

impl IndexFormat {
    /// Get an interator over all supported index formats.
    pub fn all() -> impl IntoIterator<Item = Self> {
        [
            Self::Jmdict,
            Self::Jmnedict,
            Self::Kanjidic2,
            Self::Kradfile,
        ]
    }

    /// Get the identifier of an index format.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Jmdict => "jmdict",
            Self::Jmnedict => "jmnedict",
            Self::Kanjidic2 => "kanjidic2",
            Self::Kradfile => "kradfile",
        }
    }

    /// Get a human readable short description of the format.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Jmdict => "JMDict",
            Self::Jmnedict => "Names from JMnedict",
            Self::Kanjidic2 => "Kanji from Kanjidic2",
            Self::Kradfile => "Radicals from KRADFILE",
        }
    }

    /// Generate a default index configuration for the given format.
    pub fn default_config(self, enabled: bool) -> ConfigIndex {
        match self {
            IndexFormat::Jmdict => ConfigIndex {
                format: self,
                url: JMDICT_URL.to_owned(),
                enabled,
                description: Some(JMDICT_DESCRIPTION.to_owned()),
                help: Some(JMDICT_HELP.to_owned()),
            },
            IndexFormat::Jmnedict => ConfigIndex {
                format: self,
                url: JMNEDICT_URL.to_owned(),
                enabled,
                description: Some(JMNEDICT_DESCRIPTION.to_owned()),
                help: Some(JMNEDICT_HELP.to_owned()),
            },
            IndexFormat::Kanjidic2 => ConfigIndex {
                format: self,
                url: KANJIDIC2_URL.to_owned(),
                enabled,
                description: Some(KANJIDIC2_DESCRIPTION.to_owned()),
                help: Some(KANJIDIC2_HELP.to_owned()),
            },
            IndexFormat::Kradfile => ConfigIndex {
                format: self,
                url: KRADFILE_URL.to_owned(),
                enabled,
                description: Some(KRADFILE_DESCRIPTION.to_owned()),
                help: Some(KRADFILE_HELP.to_owned()),
            },
        }
    }
}

impl FromStr for IndexFormat {
    type Err = IndexFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "jmdict" => Ok(Self::Jmdict),
            "jmnedict" => Ok(Self::Jmnedict),
            "kanjidic2" => Ok(Self::Kanjidic2),
            "kradfile" => Ok(Self::Kradfile),
            _ => Err(IndexFormatError),
        }
    }
}

/// An index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigIndex {
    pub format: IndexFormat,
    pub url: String,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

/// A configuration used for the application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Enabled indexes.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub indexes: BTreeMap<String, ConfigIndex>,
    /// Whether OCR support is enabled or not.
    #[serde(default = "default_ocr")]
    pub ocr: bool,
}

fn default_ocr() -> bool {
    true
}

impl Config {
    pub fn load(dirs: &Dirs) -> Result<Self> {
        let config_path = dirs.config_path();

        let mut config = if config_path.exists() {
            let data = std::fs::read_to_string(&config_path)?;
            toml::from_str(&data)?
        } else {
            Self::default()
        };

        for format in IndexFormat::all() {
            if !config.indexes.contains_key(format.id()) {
                config
                    .indexes
                    .insert(format.id().to_owned(), format.default_config(false));
            }
        }

        Ok(config)
    }

    /// Toggle the specified index kind.
    pub fn toggle(&mut self, id: &str) {
        if let Some(index) = self.indexes.get_mut(id) {
            index.enabled = !index.enabled;
        }
    }

    /// Test if the given index is enabled.
    pub fn is_enabled(&self, id: &str) -> bool {
        let Some(index) = self.indexes.get(id) else {
            return false;
        };

        index.enabled
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut indexes = BTreeMap::new();

        for format in IndexFormat::all() {
            indexes.insert(format.id().to_owned(), format.default_config(true));
        }

        Self { indexes, ocr: true }
    }
}
