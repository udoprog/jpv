use std::collections::BTreeSet;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::Dirs;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IndexKind {
    Jmdict,
    Jmnedict,
    Kanjidic2,
}

impl IndexKind {
    pub const ALL: &'static [IndexKind] =
        &[IndexKind::Jmdict, IndexKind::Jmnedict, IndexKind::Kanjidic2];

    /// Convert a string into an [`IndexKind`].
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "jmdict" => Some(IndexKind::Jmdict),
            "jmnedict" => Some(IndexKind::Jmnedict),
            "kanjidic2" => Some(IndexKind::Kanjidic2),
            _ => None,
        }
    }

    /// Get the name of the index.
    pub fn name(&self) -> &'static str {
        match self {
            IndexKind::Jmdict => "jmdict",
            IndexKind::Jmnedict => "jmnedict",
            IndexKind::Kanjidic2 => "kanjidic2",
        }
    }

    /// Get the name of the index.
    pub fn description(&self) -> &'static str {
        match self {
            IndexKind::Jmdict => "Phrases from JMDict",
            IndexKind::Kanjidic2 => "Kanji from Kanjidic2",
            IndexKind::Jmnedict => "Names from JMnedict",
        }
    }
}

/// An index.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    pub kind: IndexKind,
}

/// A configuration used for the application.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Enabled indexes.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub enabled: BTreeSet<IndexKind>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: IndexKind::ALL.iter().copied().collect(),
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

    /// Toggle the specified index kind.
    pub fn toggle(&mut self, kind: IndexKind) {
        if self.enabled.contains(&kind) {
            self.enabled.remove(&kind);
        } else {
            self.enabled.insert(kind);
        }
    }

    /// Test if the given index is enabled.
    pub fn is_enabled(&self, kind: IndexKind) -> bool {
        self.enabled.contains(&kind)
    }

    /// Test if the given index is enabled.
    pub fn is_enabled_by_name(&self, name: &str) -> bool {
        let Some(kind) = IndexKind::from_str(name) else {
            return false;
        };

        self.is_enabled(kind)
    }
}
