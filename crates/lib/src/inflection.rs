use std::fmt;
use std::ops::{BitAndAssign, BitOr};
use std::{collections::BTreeMap, ops::BitXor};

use fixed_map::{Key, Set};
use musli::{Decode, Encode};

use crate::kana::{Pair, Word};

/// Helper to construct a particular [`Inflection`].
///
/// # Examples
///
/// ```rust
/// lib::inflect!(Present + Past);
/// lib::inflect!(Present + Past + Polite);
/// lib::inflect!(Present + Past + Alternate);
/// ```
#[macro_export]
macro_rules! inflect {
    ($($form:ident),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut form = $crate::macro_support::fixed_map::Set::new();
        $(form.insert($crate::Form::$form);)*
        $crate::Inflection::new(form)
    }}
}

/// Helper macro to build a kana pair.
macro_rules! pair {
    ($k:expr, $r:expr, $last:expr) => {
        $crate::kana::Pair::new([$k], [$r], [$last])
    };

    ($k:expr, $r:expr, $a:expr, $last:expr) => {
        $crate::kana::Pair::new([$k, $a], [$r, $a], [$last])
    };

    ($k:expr, $r:expr, $a:expr, $b:expr, $last:expr) => {
        $crate::kana::Pair::new([$k, $a], [$r, $b], [$last])
    };
}

/// Setup a collection of inflections.
macro_rules! inflections {
    ($k:expr, $r:expr, $(
        $($kind:ident),* $(,)? ( $($tt:tt)* )
    ),* $(,)?) => {{
        let mut tree = ::std::collections::BTreeMap::new();
        $(tree.insert($crate::inflect!($($kind),*), pair!($k, $r, $($tt)*));)*
        tree
    }};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode, Key)]
#[key(bitset)]
pub enum Form {
    Te,
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
    /// te-kuru form
    Kuru,
    Polite,
    Alternate,
    Conversation,
}

impl Form {
    pub const ALL: [Form; 19] = [
        Form::Te,
        Form::Negative,
        Form::Past,
        Form::Command,
        Form::Hypothetical,
        Form::Conditional,
        Form::Passive,
        Form::Potential,
        Form::Volitional,
        Form::Causative,
        Form::Tai,
        Form::Progressive,
        Form::Resulting,
        Form::Iku,
        Form::Shimau,
        Form::Kuru,
        Form::Polite,
        Form::Alternate,
        Form::Conversation,
    ];

    /// Longer title for the form.
    pub fn title(&self) -> &'static str {
        match self {
            Form::Te => "~te form, by itself acts as a command",
            Form::Negative => "not doing ~, the absense of ~",
            Form::Past => "past tense",
            Form::Command => "command form",
            Form::Hypothetical => "if ~",
            Form::Conditional => "if ~, when ~",
            Form::Passive => "~ was done to someone or something",
            Form::Potential => "can do ~",
            Form::Volitional => "let's do ~",
            Form::Causative => "make (someone do something), let / allow (someone to do something)",
            Form::Tai => "used to express desire",
            Form::Progressive => "~te iru, shows that something is currently happening or ongoing",
            Form::Resulting => "~te aru, is/has been done (resulting state)",
            Form::Iku => "~te iku, to start, to continue, to go on",
            Form::Shimau => "~te shimau, to do something by accident, to finish completely",
            Form::Kuru => "~te kuru, to do .. and come back, to become, to continue, to start ~",
            Form::Polite => "polite form",
            Form::Alternate => "alternate form",
            Form::Conversation => "conversational use only",
        }
    }

    /// Describe the form.
    pub fn describe(&self) -> &'static str {
        match self {
            Form::Te => "~te",
            Form::Negative => "negative",
            Form::Past => "past",
            Form::Command => "command",
            Form::Hypothetical => "hypothetical",
            Form::Conditional => "conditional",
            Form::Passive => "passive",
            Form::Potential => "potential",
            Form::Volitional => "volitional",
            Form::Causative => "causative",
            Form::Tai => "~tai form",
            Form::Progressive => "~te iru",
            Form::Resulting => "~te aru,",
            Form::Iku => "~te iku",
            Form::Shimau => "~te shimau",
            Form::Kuru => "~te kuru",
            Form::Polite => "polite",
            Form::Alternate => "alt",
            Form::Conversation => "conversational",
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode)]
pub struct Inflection {
    #[musli(with = crate::musli::set::<_>)]
    pub form: Set<Form>,
}

impl Inflection {
    // Macro support.
    #[doc(hidden)]
    pub fn new(form: Set<Form>) -> Self {
        Self { form }
    }

    // Construct an inflection with all options set.
    pub fn all() -> Self {
        let mut form = Set::new();

        for f in Form::ALL {
            form.insert(f);
        }

        Self { form }
    }

    /// Toggle the given form.
    pub fn toggle(&mut self, form: Form) {
        if self.form.contains(form) {
            self.form.remove(form);
        } else {
            self.form.insert(form);
        }
    }
}

impl fmt::Debug for Inflection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.form.fmt(f)
    }
}

impl BitOr for Inflection {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            form: Set::from_raw(self.form.as_raw() | rhs.form.as_raw()),
        }
    }
}

impl BitXor for Inflection {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            form: Set::from_raw(self.form.as_raw() ^ rhs.form.as_raw()),
        }
    }
}

impl BitAndAssign for Inflection {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.form = Set::from_raw(self.form.as_raw() & rhs.form.as_raw());
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
            if c.form.contains(Form::Polite) {
                return true;
            }
        }

        false
    }

    /// Test if an inflection exists.
    pub fn contains(&self, inflection: Inflection) -> bool {
        self.inflections.contains_key(&inflection)
    }

    /// Get a inflection.
    pub fn get(&self, inflection: Inflection) -> Option<&Pair<'a, 3, 4>> {
        self.inflections.get(&inflection)
    }

    /// Iterate over all inflections.
    pub fn iter(&self) -> impl Iterator<Item = (&Inflection, &Pair<'a, 3, 4>)> + '_ {
        self.inflections.iter()
    }
}
