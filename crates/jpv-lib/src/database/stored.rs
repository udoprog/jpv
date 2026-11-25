use musli_zerocopy::endian::Little;
use musli_zerocopy::{slice, swiss, trie, ByteOrder, Ref, ZeroCopy};

use crate::PartOfSpeech;

use super::{KanjiIndex, NameIndex, PhraseIndex};

pub struct CompactTrie;

impl trie::Flavor for CompactTrie {
    type String<E>
        = slice::Packed<[u8], E, u32, u8>
    where
        E: ByteOrder;
    type Values<T, E>
        = slice::Packed<[T], E, u32, u16>
    where
        T: ZeroCopy,
        E: ByteOrder;
    type Children<T, E>
        = slice::Packed<[T], E, u32, u16>
    where
        T: ZeroCopy,
        E: ByteOrder;
}

#[derive(ZeroCopy)]
#[repr(C)]
pub(super) struct GlobalHeader {
    pub(super) magic: u32,
    pub(super) version: u32,
    pub(super) index: Ref<IndexHeader, Little>,
}

#[derive(Clone, Copy, ZeroCopy)]
#[repr(C)]
pub(super) struct IndexHeader {
    pub(super) name: Ref<str>,
    pub(super) lookup: trie::TrieRef<Id, Little, CompactTrie>,
    /// Phrases by position.
    pub(super) by_pos: swiss::MapRef<PartOfSpeech, Ref<[PhrasePos]>>,
    /// Kanjis by literal.
    pub(super) by_kanji_literal: swiss::MapRef<Ref<str>, u32>,
    pub(super) radicals: swiss::MapRef<Ref<str>, u32>,
    pub(super) radicals_to_kanji: swiss::MapRef<Ref<str>, Ref<[u32]>>,
    pub(super) by_sequence: swiss::MapRef<u32, PhrasePos>,
    /// The offset of all phrases stored in the index.
    pub(super) phrases: Ref<[u32]>,
    /// The offset of all kanji stored in the index.
    pub(super) kanji: Ref<[u32]>,
}

/// Extra information about an index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
#[repr(u8)]
pub(super) enum Source {
    /// Indexed due to a kanji.
    Kanji { index: KanjiIndex },
    /// Indexed due to a phrase.
    Phrase { index: PhraseIndex },
    /// Indexed to to a name.
    Name { index: NameIndex },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
#[repr(C)]
pub(super) struct PhrasePos {
    pub(super) offset: u32,
    pub(super) reading: PhraseIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
#[repr(C)]
pub(super) struct Id {
    pub(super) offset: u32,
    pub(super) source: Source,
}

impl Id {
    pub(super) fn phrase(offset: u32, index: PhraseIndex) -> Self {
        Self {
            offset,
            source: Source::Phrase { index },
        }
    }

    pub(super) fn name(offset: u32, index: NameIndex) -> Self {
        Self {
            offset,
            source: Source::Name { index },
        }
    }

    pub(super) fn kanji(offset: u32, index: KanjiIndex) -> Self {
        Self {
            offset,
            source: Source::Kanji { index },
        }
    }
}
