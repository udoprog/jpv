use musli::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode)]
pub enum Priority {
    /// Common words.
    Ichi(u8),
    /// News.
    News(u8),
    /// Common loan words.
    Gai(u8),
    /// Especially marked common words.
    Spec(u8),
    /// Word frequency category.
    WordFrequency(u8),
}

impl Priority {
    /// Parse a priority.
    pub fn parse(string: &str) -> Option<Priority> {
        let n = string.find(char::is_numeric)?;
        let group = string[n..].parse().ok()?;

        match &string[..n] {
            "ichi" => Some(Priority::Ichi(group)),
            "news" => Some(Priority::News(group)),
            "gai" => Some(Priority::Gai(group)),
            "spec" => Some(Priority::Spec(group)),
            "nf" => Some(Priority::WordFrequency(group)),
            _ => None,
        }
    }

    /// Priority level.
    pub fn level(&self) -> usize {
        match *self {
            Priority::Ichi(n) => n as usize,
            Priority::News(n) => n as usize,
            Priority::Gai(n) => n as usize,
            Priority::Spec(n) => n as usize,
            Priority::WordFrequency(n) => n as usize,
        }
    }

    /// Get priority category.
    pub fn category(&self) -> &str {
        match self {
            Priority::Ichi(..) => "ichi",
            Priority::News(..) => "news",
            Priority::Gai(..) => "gai",
            Priority::Spec(..) => "spec",
            Priority::WordFrequency(..) => "nf",
        }
    }
}
