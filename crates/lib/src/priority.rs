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
}
