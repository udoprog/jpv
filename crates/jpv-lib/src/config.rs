use std::collections::BTreeMap;
use std::str::FromStr;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Dirs;

const JMDICT_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/JMdict_e_examp.gz";
const KANJIDIC2_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/kanjidic2.xml.gz";
const JMNEDICT_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/JMnedict.xml.gz";

#[derive(Debug, Error)]
#[error("Invalid index format")]
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
}

impl FromStr for IndexFormat {
    type Err = IndexFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "jmdict" => Ok(Self::Jmdict),
            "jmnedict" => Ok(Self::Jmnedict),
            "kanjidic2" => Ok(Self::Kanjidic2),
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

        indexes.insert(
            "jmdict".to_owned(),
            ConfigIndex {
                format: IndexFormat::Jmdict,
                url: JMDICT_URL.to_owned(),
                enabled: true,
                description: Some("JMDict (with examples)".to_owned()),
                help: Some(
                    "https://www.edrdg.org/wiki/index.php/JMdict-EDICT_Dictionary_Project"
                        .to_owned(),
                ),
            },
        );

        indexes.insert(
            "jmnedict".to_owned(),
            ConfigIndex {
                format: IndexFormat::Jmnedict,
                url: JMNEDICT_URL.to_owned(),
                enabled: true,
                description: Some("Names from JMnedict".to_owned()),
                help: Some(
                    "https://www.edrdg.org/wiki/index.php/Main_Page#The_ENAMDICT/JMnedict_Project"
                        .to_owned(),
                ),
            },
        );

        indexes.insert(
            "kanjidic2".to_owned(),
            ConfigIndex {
                format: IndexFormat::Kanjidic2,
                url: KANJIDIC2_URL.to_owned(),
                enabled: true,
                description: Some("Kanji from Kanjidic2".to_owned()),
                help: Some("https://www.edrdg.org/wiki/index.php/KANJIDIC_Project".to_owned()),
            },
        );

        Self { indexes }
    }
}
