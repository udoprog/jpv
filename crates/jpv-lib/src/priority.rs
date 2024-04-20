use std::fmt;

use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "kebab-case")]
#[musli(mode = Text, name_all = "kebab-case")]
pub enum PriorityKind {
    /// Common words.
    Ichi,
    /// News.
    News,
    /// Common loan words.
    Gai,
    /// Especially marked common words.
    Spec,
    /// Word frequency category.
    #[serde(rename = "nf")]
    #[musli(mode = Text, name = "nf")]
    WordFrequency,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Encode, Decode)]
#[musli(mode = Binary, packed)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct Priority {
    level: u8,
    kind: PriorityKind,
}

impl Priority {
    /// Parse a priority.
    pub fn parse(string: &str) -> Option<Priority> {
        let n = string.find(char::is_numeric)?;
        let level = string[n..].parse().ok()?;

        let kind = match &string[..n] {
            "ichi" => PriorityKind::Ichi,
            "news" => PriorityKind::News,
            "gai" => PriorityKind::Gai,
            "spec" => PriorityKind::Spec,
            "nf" => PriorityKind::WordFrequency,
            _ => return None,
        };

        Some(Priority { level, kind })
    }

    /// Priority level.
    pub fn level(&self) -> usize {
        self.level as usize
    }

    /// Get priority category.
    pub fn category(&self) -> &'static str {
        match self.kind {
            PriorityKind::Ichi => "ichi",
            PriorityKind::News => "news",
            PriorityKind::Gai => "gai",
            PriorityKind::Spec => "spec",
            PriorityKind::WordFrequency => "nf",
        }
    }

    pub fn title(&self) -> &'static str {
        match self.kind {
            PriorityKind::Ichi => {
                "appears in \"Ichimango goi bunruishuu\", ichi2 are less frequently used online"
            }
            PriorityKind::News => "frequently used in news",
            PriorityKind::Gai => "common loanwords",
            PriorityKind::Spec => "special words",
            PriorityKind::WordFrequency => "word frequency, lower means more frequent",
        }
    }

    /// Weight for these priorities.
    pub(crate) fn weight(&self) -> f32 {
        let level = self.level.saturating_sub(1) as f32;

        // Calculate the range-based priority.
        macro_rules! range {
            ($max:expr) => {
                1.0 + ($max - level.min($max)) / $max
            };
        }

        match self.kind {
            PriorityKind::Ichi => range!(2.0) * 2.0,
            PriorityKind::News => range!(2.0),
            PriorityKind::Gai => range!(2.0),
            PriorityKind::Spec => range!(2.0) * 2.2,
            PriorityKind::WordFrequency => range!(50.0) * 2.0,
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.category(), self.level)
    }
}
