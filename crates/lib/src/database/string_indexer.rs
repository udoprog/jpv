use std::collections::{btree_map, BTreeMap};

use musli_zerocopy::{Error, OwnedBuf, Ref};

/// Index a collection of strings, re-using storage for any subsequent
/// sub-strings which are already registered.
///
/// This achieves optimal string-reuse only if a collection of strings are
/// provided in ascending byte sorted order.
///
/// Since strings are compared lexicographically by their byte order by default,
/// all you need to do is reverse sort a collection of strings:
///
/// ```
/// let mut strings: Vec<&str> = Vec::new();
/// strings.sort_by(|a, b| a.cmp(b).reverse());
/// ```
pub struct StringIndexer<'a> {
    existing: BTreeMap<&'a str, usize>,
    reuse: usize,
    total: usize,
}

impl<'a> StringIndexer<'a> {
    pub fn new() -> Self {
        Self {
            existing: BTreeMap::new(),
            reuse: 0,
            total: 0,
        }
    }

    /// Get the number of re-used strings in the collection.
    pub fn reuse(&self) -> usize {
        self.reuse
    }

    /// Get the number of total strings in the collection.
    pub fn total(&self) -> usize {
        self.total
    }

    fn find(&self, input: &str) -> Option<Ref<str>> {
        let (&key, &offset) = self.existing.range(input..).next()?;

        if !key.starts_with(input) {
            return None;
        }

        Some(Ref::with_metadata(offset, input.len()))
    }

    /// Store the given string while possibly re-using storage for it, returning
    /// the string reference it belongs to.
    pub(crate) fn store(&mut self, buf: &mut OwnedBuf, input: &'a str) -> Result<Ref<str>, Error> {
        self.total += 1;

        if let Some(r) = self.find(input) {
            self.reuse += 1;
            debug_assert_eq!(buf.load(r)?, input);
            return Ok(r);
        }

        let r = buf.store_unsized(input);
        let mut o = r.offset();

        if let btree_map::Entry::Vacant(e) = self.existing.entry(input) {
            e.insert(o);
        }

        let mut it = input.chars();

        while let Some(c) = it.next() {
            o += c.len_utf8();

            if self.find(it.as_str()).is_some() {
                continue;
            }

            self.existing.insert(it.as_str(), o);
        }

        Ok(r)
    }
}
