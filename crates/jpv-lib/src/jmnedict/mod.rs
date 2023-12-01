pub use self::parser::Parser;
pub mod parser;

pub use self::elements::{Entry, OwnedEntry, OwnedReading, OwnedTranslation, Reading, Translation};
mod elements;
