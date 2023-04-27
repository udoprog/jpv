use musli::{Decode, Encode};

#[derive(Debug, Clone, Copy, Encode, Decode)]
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
    WordFrequency,
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
#[musli(packed)]
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
    pub fn category(&self) -> &str {
        match self.kind {
            PriorityKind::Ichi => "ichi",
            PriorityKind::News => "news",
            PriorityKind::Gai => "gai",
            PriorityKind::Spec => "spec",
            PriorityKind::WordFrequency => "nf",
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
