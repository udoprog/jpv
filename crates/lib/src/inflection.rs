use std::fmt;
use std::ops::{BitAndAssign, BitOr};
use std::{collections::BTreeMap, ops::BitXor};

use fixed_map::{Key, Set};
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::kana::{Fragments, Full, OwnedFull};

/// Helper to construct a particular [`Inflection`].
///
/// # Examples
///
/// ```rust
/// lib::inflect!(Past);
/// lib::inflect!(Past, Polite);
/// lib::inflect!(Past, Alternate);
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
        $crate::kana::Fragments::new([$k], [$r], [$last])
    };

    ($k:expr, $r:expr, $a:expr, $last:expr) => {
        $crate::kana::Fragments::new([$k, $a], [$r, $a], [$last])
    };

    ($k:expr, $r:expr, $a:expr, $b:expr, $last:expr) => {
        $crate::kana::Fragments::new([$k, $a], [$r, $b], [$last])
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
    /// Te-form.
    Te,
    /// Te-iru or progressive form.
    TeIru,
    /// Te-aru or resulting form.
    TeAru,
    /// Te-iku form.
    TeIku,
    /// te-shimau form
    TeShimau,
    /// chau form
    Chau,
    /// te-kuru form
    TeKuru,
    /// te-oku form
    TeOku,
    Command,
    Hypothetical,
    Conditional,
    Passive,
    Potential,
    /// Volitional / Presumptive
    Volitional,
    Causative,
    Tai,
    Negative,
    Past,
    Polite,
    Conversation,
    Alternate,
}

impl Form {
    pub const ALL: [Form; 21] = [
        Form::Alternate,
        Form::Causative,
        Form::Chau,
        Form::Command,
        Form::Conditional,
        Form::Conversation,
        Form::Hypothetical,
        Form::Negative,
        Form::Passive,
        Form::Past,
        Form::Polite,
        Form::Potential,
        Form::Tai,
        Form::Te,
        Form::TeAru,
        Form::TeIku,
        Form::TeIru,
        Form::TeKuru,
        Form::TeOku,
        Form::TeShimau,
        Form::Volitional,
    ];

    /// Longer title for the form.
    pub fn title(&self) -> &'static str {
        match self {
            Form::Alternate => "alternate form",
            Form::Causative => "causative, make ~ do something, let / allow ~",
            Form::Chau => "chau, to do something by accident, to finish completely",
            Form::Command => "command",
            Form::Conditional => "conditional, if ~, when ~",
            Form::Conversation => "conversational use only",
            Form::Hypothetical => "hypothetical, if ~",
            Form::Negative => "not doing ~, the absense of ~",
            Form::Passive => "passive, ~ was done to someone or something",
            Form::Past => "past tense",
            Form::Polite => "polite form",
            Form::Potential => "potential, can do ~",
            Form::Tai => "tai-form, used to express desire",
            Form::Te => "~te form, by itself acts as a command",
            Form::TeAru => "~te aru, resulting, is/has been done",
            Form::TeIku => "~te iku, starting, to start, to continue, to go on",
            Form::TeIru => {
                "~te iru, progressive, shows that something is currently happening or ongoing"
            }
            Form::TeKuru => "~te kuru, to do .. and come back, to become, to continue, to start ~",
            Form::TeOku => "~te oku, to do something in advance",
            Form::TeShimau => "~te shimau, to do something by accident, to finish completely",
            Form::Volitional => "volitional / presumptive, let's do ~",
        }
    }

    /// Describe the form.
    pub fn describe(&self) -> &'static str {
        match self {
            Form::Alternate => "alternate",
            Form::Causative => "causative",
            Form::Chau => "chau",
            Form::Command => "command",
            Form::Conditional => "conditional",
            Form::Conversation => "conversation",
            Form::Hypothetical => "hypothetical",
            Form::Negative => "negative",
            Form::Passive => "passive",
            Form::Past => "past",
            Form::Polite => "polite",
            Form::Potential => "potential",
            Form::Tai => "~tai",
            Form::Te => "~te",
            Form::TeAru => "~te aru,",
            Form::TeIku => "~te iku",
            Form::TeIru => "~te iru",
            Form::TeKuru => "~te kuru",
            Form::TeOku => "~te oku",
            Form::TeShimau => "~te shimau",
            Form::Volitional => "volitional",
        }
    }
}

#[derive(
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
pub struct Inflection {
    #[musli(with = crate::musli::set::<_>)]
    #[serde(with = "crate::serde::set")]
    form: Set<Form>,
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

    /// Test if inflection is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.form.is_empty()
    }

    /// Test if inflection contains the given form.
    #[inline]
    pub fn contains(&self, f: Form) -> bool {
        self.form.contains(f)
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = Form> {
        self.form.iter()
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
#[owned::owned]
pub struct Inflections<'a> {
    #[owned(ty = OwnedFull)]
    pub dictionary: Full<'a>,
    #[owned(ty = BTreeMap<Inflection, OwnedFull>, with = self::inflections)]
    pub inflections: BTreeMap<Inflection, Fragments<'a, 3, 4>>,
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
    pub fn get(&self, inflection: Inflection) -> Option<&Fragments<'a, 3, 4>> {
        self.inflections.get(&inflection)
    }

    /// Iterate over all inflections.
    pub fn iter(&self) -> impl Iterator<Item = (&Inflection, &Fragments<'a, 3, 4>)> + '_ {
        self.inflections.iter()
    }
}

impl OwnedInflections {
    /// Test if an inflection exists.
    pub fn contains(&self, inflection: Inflection) -> bool {
        self.inflections.contains_key(&inflection)
    }

    /// Get a inflection.
    pub fn get(&self, inflection: Inflection) -> Option<&OwnedFull> {
        self.inflections.get(&inflection)
    }
}

mod inflections {
    use std::collections::BTreeMap;

    use crate::kana::{Fragments, OwnedFull};
    use crate::Inflection;

    pub(crate) fn to_owned(
        this: &BTreeMap<Inflection, Fragments<'_, 3, 4>>,
    ) -> BTreeMap<Inflection, OwnedFull> {
        let mut out = BTreeMap::new();

        for (key, value) in this {
            out.insert(
                *key,
                OwnedFull {
                    text: value.text().to_string(),
                    reading: value.reading().to_string(),
                    suffix: value.suffix().to_string(),
                },
            );
        }

        out
    }

    pub(crate) fn borrow(
        this: &BTreeMap<Inflection, OwnedFull>,
    ) -> BTreeMap<Inflection, Fragments<'_, 3, 4>> {
        let mut out = BTreeMap::new();

        for (key, value) in this {
            out.insert(
                *key,
                Fragments::new(
                    [value.text.as_str()],
                    [value.reading.as_str()],
                    [value.suffix.as_str()],
                ),
            );
        }

        out
    }
}
