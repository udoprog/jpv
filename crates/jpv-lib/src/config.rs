use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::Dirs;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IndexKind {
    Jmdict,
    Kanjidic2,
    Jmnedict,
}

impl IndexKind {
    pub const ALL: &'static [IndexKind] =
        &[IndexKind::Jmdict, IndexKind::Kanjidic2, IndexKind::Jmnedict];

    /// Get the name of the index.
    pub fn name(&self) -> &str {
        match self {
            IndexKind::Jmdict => "jmdict",
            IndexKind::Kanjidic2 => "kanjidic2",
            IndexKind::Jmnedict => "jmnedict",
        }
    }

    /// Get the name of the index.
    pub fn description(&self) -> &str {
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
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    /// Enabled indexes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub indexes: Vec<Index>,
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

    /// Test if the given index is enabled.
    pub fn is_enabled(&self, name: &str) -> bool {
        for index in &self.indexes {
            if name == index.kind.name() {
                return true;
            }
        }

        false
    }
}
