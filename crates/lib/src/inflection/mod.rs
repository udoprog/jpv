mod godan;
#[macro_use]
mod macros;

pub use self::conjugate::{conjugate, Kind, Reading};
mod conjugate;

use std::fmt;
use std::ops::{BitAndAssign, BitOr};
use std::{collections::BTreeMap, ops::BitXor};

use fixed_map::raw::RawStorage;
use fixed_map::{Key, Set};
use musli::{Decode, Encode};
use musli_zerocopy::buf::{Padder, Validator};
use musli_zerocopy::ZeroCopy;
use serde::{Deserialize, Serialize};

use crate::kana::{Fragments, Full, OwnedFull};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Encode,
    Decode,
    Serialize,
    Deserialize,
    Key,
)]
#[key(bitset)]
#[serde(rename_all = "kebab-case")]
pub enum Form {
    /// The stem of the word.
    Stem,
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
    /// Alternate negative hypoethical form.
    Kya,
    Conditional,
    Passive,
    Potential,
    /// Volitional / Presumptive
    Volitional,
    Causative,
    Tai,
    Negative,
    Past,
    /// Polite form.
    Polite,
    /// Conversational form.
    Conversation,
    /// Alternate forms, when available.
    Short,
    /// Alternate form using kudasai.
    Kudasai,
    /// Alternate form using darou.
    Darou,
    /// Alternate command form using yo.
    Yo,
}

impl Form {
    pub const ALL: [Form; 26] = [
        Form::Stem,
        Form::Short,
        Form::Causative,
        Form::Chau,
        Form::Command,
        Form::Conditional,
        Form::Conversation,
        Form::Hypothetical,
        Form::Kya,
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
        Form::Kudasai,
        Form::Darou,
        Form::Yo,
    ];

    /// Longer title for the form.
    pub fn title(&self) -> &'static str {
        match self {
            Form::Stem => "stem, or infinite form",
            Form::Short => "alternate shortened form",
            Form::Causative => "causative, make ~ do something, let / allow ~",
            Form::Chau => "to do something by accident, to finish completely",
            Form::Command => "command",
            Form::Conditional => "conditional, if ~, when ~",
            Form::Conversation => "conversational use only",
            Form::Hypothetical => "hypothetical, if ~",
            Form::Kya => "~kya, alternative hypothetical negative, if not ~",
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
            Form::Kudasai => "alternate form using ~kudasai",
            Form::Darou => "alternate form using ~darou / ~deshou",
            Form::Yo => "alternate command form using ~yo",
        }
    }

    /// Describe the form.
    pub fn describe(&self) -> &'static str {
        match self {
            Form::Stem => "stem / infinite",
            Form::Short => "short",
            Form::Causative => "causative",
            Form::Chau => "~chau / ~jau",
            Form::Command => "command",
            Form::Conditional => "conditional",
            Form::Conversation => "conversation",
            Form::Hypothetical => "hypothetical",
            Form::Kya => "~kya",
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
            Form::Kudasai => "kudasai",
            Form::Darou => "~darou / ~deshou",
            Form::Yo => "~yo",
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
#[serde(transparent)]
#[musli(transparent)]
#[repr(transparent)]
pub struct Inflection {
    #[musli(with = crate::musli::set::<_>)]
    form: Set<Form>,
}

unsafe impl ZeroCopy for Inflection
where
    <<Form as Key>::SetStorage as RawStorage>::Value: ZeroCopy,
{
    const ANY_BITS: bool = false;
    const PADDED: bool = false;

    unsafe fn validate(v: &mut Validator<'_, Self>) -> Result<(), musli_zerocopy::Error> {
        <<Form as Key>::SetStorage as RawStorage>::Value::validate(v.transparent())
    }

    unsafe fn pad(p: &mut Padder<'_, Self>) {
        <<Form as Key>::SetStorage as RawStorage>::Value::pad(p.transparent())
    }
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
#[borrowme::borrowme]
pub struct Inflections<'a> {
    pub dictionary: Full<'a>,
    #[borrowme(owned = BTreeMap<Inflection, OwnedFull>, with = self::inflections)]
    pub inflections: BTreeMap<Inflection, Fragments<'a, 3, 4>>,
}

impl<'a> Inflections<'a> {
    pub fn new(dictionary: Full<'a>) -> Self {
        Self {
            dictionary,
            inflections: BTreeMap::new(),
        }
    }

    /// Insert a value into this collection of inflections.
    pub(crate) fn insert(
        &mut self,
        inflect: &[Form],
        inflect2: &[Form],
        word: Fragments<'a, 3, 4>,
    ) {
        let mut form = crate::macro_support::fixed_map::Set::new();

        for f in inflect {
            form.insert(*f);
        }

        for f in inflect2 {
            form.insert(*f);
        }

        self.inflections.insert(crate::Inflection::new(form), word);
    }

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

    pub(crate) fn to_owned<const N: usize, const S: usize>(
        this: &BTreeMap<Inflection, Fragments<'_, N, S>>,
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

    pub(crate) fn borrow<const N: usize, const S: usize>(
        this: &BTreeMap<Inflection, OwnedFull>,
    ) -> BTreeMap<Inflection, Fragments<'_, N, S>> {
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
