pub use self::parser::Parser;
mod parser;

pub use self::elements::{Entry, OwnedEntry};
pub use self::elements::{Example, OwnedExample};
pub use self::elements::{ExampleSentence, OwnedExampleSentence};
pub use self::elements::{ExampleSource, OwnedExampleSource};
pub use self::elements::{Glossary, OwnedGlossary};
pub use self::elements::{KanjiElement, OwnedKanjiElement};
pub use self::elements::{OwnedReadingElement, ReadingElement};
pub use self::elements::{OwnedSense, Sense};
pub use self::elements::{OwnedSourceLanguage, SourceLanguage};
pub(crate) mod elements;
