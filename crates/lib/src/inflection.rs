use std::collections::BTreeMap;
use std::fmt;
use std::ops::BitOr;

use fixed_map::{Key, Set};
use musli::{Decode, Encode};

use crate::kana::{Pair, Word};
use crate::Concat;

/// Helper to construct a particular [`Inflection`].
///
/// # Examples
///
/// ```rust
/// lib::inflect!(Present + Past);
/// lib::inflect!(Present + Past + *Polite);
/// lib::inflect!(Present + Past + *Alternate);
/// ```
#[macro_export]
macro_rules! inflect {
    ($kind:ident $(+ $kind2:ident)* $(+ *$flag:ident)*) => {{
        let mut form = $crate::macro_support::fixed_map::Set::new();
        form.insert($crate::Form::$kind);
        $(form.insert($crate::Form::$kind2);)*
        #[allow(unused_mut)]
        let mut flag = $crate::macro_support::fixed_map::Set::new();
        $(flag.insert($crate::Flag::$flag);)*
        $crate::Inflection::new(form, flag)
    }}
}

/// Helper macro to build a kana pair.
macro_rules! pair {
    ($k:expr, $r:expr, $last:expr) => {
        $crate::kana::Pair::new([$k], [$r], $last)
    };

    ($k:expr, $r:expr, $a:expr, $last:expr) => {
        $crate::kana::Pair::new([$k, $a], [$r, $a], $last)
    };

    ($k:expr, $r:expr, $a:expr, $b:expr, $last:expr) => {
        $crate::kana::Pair::new([$k, $a], [$r, $b], $last)
    };
}

/// Setup a collection of inflections.
macro_rules! inflections {
    ($k:expr, $r:expr, $(
        $kind:ident $(+ $kind2:ident)* $(+ *$flag:ident)* ( $($tt:tt)* )
    ),* $(,)?) => {{
        let mut tree = ::std::collections::BTreeMap::new();
        $(tree.insert($crate::inflect!($kind $(+ $kind2)* $(+ *$flag)*), pair!($k, $r, $($tt)*));)*
        tree
    }};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode, Key)]
#[key(bitset)]
pub enum Form {
    Te,
    Present,
    Negative,
    Past,
    Command,
    Hypothetical,
    Conditional,
    Passive,
    Potential,
    /// Volitional / Presumptive
    Volitional,
    Causative,
    Tai,
    /// Te-iru or progressive form.
    Progressive,
    /// Te-aru or resulting form.
    Resulting,
    /// Te-iku form.
    Iku,
    /// te-shimau form
    Shimau,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode, Key)]
#[key(bitset)]
pub enum Flag {
    Polite,
    Alternate,
    Conversation,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode)]
pub struct Inflection {
    #[musli(with = crate::musli::set::<_>)]
    pub form: Set<Form>,
    #[musli(with = crate::musli::set::<_>)]
    pub flag: Set<Flag>,
}

impl Inflection {
    // Macro support.
    #[doc(hidden)]
    pub fn new(form: Set<Form>, flag: Set<Flag>) -> Self {
        Self { form, flag }
    }
}

impl fmt::Debug for Inflection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.flag.is_empty() {
            self.form.fmt(f)
        } else {
            write!(f, "{:?} / {:?}", self.form, self.flag)
        }
    }
}

impl BitOr for Inflection {
    type Output = Self;

    #[inline]
    fn bitor(mut self, rhs: Self) -> Self::Output {
        for f in rhs.form {
            self.form.insert(f);
        }

        for f in rhs.flag {
            self.flag.insert(f);
        }

        self
    }
}

/// A collection of inflections.
#[non_exhaustive]
pub struct Inflections<'a> {
    pub dictionary: Word<'a>,
    pub inflections: BTreeMap<Inflection, Pair<'a, 3, 4>>,
}

impl<'a> Inflections<'a> {
    /// Test if any polite inflections exist.
    pub fn has_polite(&self) -> bool {
        for c in self.inflections.keys() {
            if c.flag.contains(Flag::Polite) {
                return true;
            }
        }

        false
    }

    /// Get a inflection.
    pub fn get(&self, inflection: Inflection) -> Option<&Pair<'a, 3, 4>> {
        self.inflections.get(&inflection)
    }

    /// Iterate over all inflections.
    pub fn iter(&self) -> impl Iterator<Item = (Inflection, Concat<'a, 6>)> + '_ {
        self.inflections
            .iter()
            .flat_map(|(k, p)| p.clone().into_iter().map(|p| (*k, p)))
    }
}
