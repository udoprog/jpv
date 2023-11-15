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
use musli_zerocopy::{ByteOrder, ZeroCopy};
use serde::{Deserialize, Serialize};

use crate::kana::{Fragments, Full, OwnedFull};

macro_rules! form {
    ($vis:vis enum $name:ident { $({$variant:ident = $d:literal, $describe:literal, $title:literal, $url:expr $(,)?}),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[derive(Encode, Decode, Serialize, Deserialize, Key)]
        #[key(bitset = 8)]
        #[serde(rename_all = "kebab-case")]
        $vis enum $name {
            $($variant = $d,)*
        }

        impl $name {
            $vis const ALL: [Form; 31] = [
                $(Form::$variant,)*
            ];

            /// Describe the form.
            $vis fn describe(&self) -> &'static str {
                match self {
                    $(Form::$variant => $describe,)*
                }
            }

            /// Longer title for the form.
            $vis fn title(&self) -> &'static str {
                match self {
                    $(Form::$variant => $title,)*
                }
            }

            /// Tutorial URL for the form.
            $vis fn url(&self) -> Option<&'static str> {
                match self {
                    $(Form::$variant => $url,)*
                }
            }
        }
    }
}

form! {
    pub enum Form {
        {Stem = 0, "stem", "stem / infinite", None},
        {Honorific = 27, "敬語", "敬語 (ていご) honorific speech", None},
        {Negative = 1, "not", "not doing ~, the absense of ~", None},
        {Te = 17, "～て", "～te form, by itself acts as a command", None},
        {TeAru = 18, "～てある", "～てある, resulting, is / has been done", None},
        {TeIku = 19, "～ていく", "～ていく, starting, to start, to continue, to go on", None},
        {TeIru = 20, "～ている", "～ている, progressive, shows that something is currently happening or ongoing", Some("https://www.tofugu.com/japanese-grammar/verb-continuous-form-teiru/")},
        {TeKuru = 21, "～てくる", "～てくる, to do .. and come back, to become, to continue, to start ~", None},
        {TeOku = 22, "～ておく", "～ておく, to do something in advance", None},
        {TeShimau = 23, "～てしまう", "～てしまう, to do something by accident, to finish completely", None},
        {Tai = 15, "～たい", "～たい, expressing desire", Some("https://www.tofugu.com/japanese-grammar/tai-form/")},
        {EasyTo = 29, "easy", "～やすい, easy to do ~", Some("https://www.tofugu.com/japanese-grammar/yasui/")},
        {HardTo = 30, "hard", "～にくい, hard to do ~", Some("https://www.tofugu.com/japanese-grammar/nikui/")},
        {TaGaRu = 16, "～たがる", "～たがる, noting desire", Some("https://www.tofugu.com/japanese-grammar/tagaru-form/")},
        {Causative = 2, "caus", "causative, make ~ do something, let / allow ~", None},
        {Chau = 3, "～ちゃう", "～ちゃう, to do something by accident, to finish completely", None},
        {Command = 4, "cmd", "command forms, よ / なさい / ください", Some("https://www.tofugu.com/japanese-grammar/verb-command-form-ro/")},
        {CommandTeKudasai = 5, "～てください", "～てください, alternate command form", None},
        {CommandYo = 6, "～よ", "～よ, alternate command form", None},
        {Conditional = 7, "cond", "～たら, conditional, if ~, when ~", Some("https://www.tofugu.com/japanese-grammar/conditional-form-tara/")},
        {Conversation = 8, "clq", "conversational / colloquial", None},
        {Darou = 9, "～だろう", "～だろう, alternate form", None},
        {Hypothetical = 10, "hyp", "hypothetical, if ~", None},
        {Kya = 11, "～きゃ", "～きゃ, alternative hypothetical negative, if not ~", None},
        {Passive = 12, "psv", "～られる, passive, ~ was done to someone or something", Some("https://www.tofugu.com/japanese-grammar/verb-passive-form-rareru/")},
        {Past = 13, "past", "過去形 (かこけい) past tense", None},
        {Potential = 14, "pot", "potential, can do ~", None},
        {Simultaneous = 28, "～ながら", "～ながら, simultaneous, while ~", Some("https://www.tofugu.com/japanese-grammar/verb-nagara/")},
        {Volitional = 24, "vol", "～よう, volitional / presumptive, let's do ~", Some("https://www.tofugu.com/japanese-grammar/verb-volitional-form-you/")},
        {Short = 25, "short", "alternate shortened form", None},
        {LooksLike = 26, "～そう", "～そう, looks like", Some("https://www.tofugu.com/japanese-grammar/verb-sou/")},
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
    const CAN_SWAP_BYTES: bool = <<Form as Key>::SetStorage as RawStorage>::Value::CAN_SWAP_BYTES;

    #[inline]
    unsafe fn validate(v: &mut Validator<'_, Self>) -> Result<(), musli_zerocopy::Error> {
        <<Form as Key>::SetStorage as RawStorage>::Value::validate(v.transparent())
    }

    #[inline]
    unsafe fn pad(p: &mut Padder<'_, Self>) {
        <<Form as Key>::SetStorage as RawStorage>::Value::pad(p.transparent())
    }

    #[inline]
    fn swap_bytes<E: ByteOrder>(self) -> Self {
        let form = <<Form as Key>::SetStorage as RawStorage>::Value::swap_bytes(self.form.as_raw());

        Inflection {
            form: Set::from_raw(form),
        }
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
    pub inflections: BTreeMap<Inflection, Fragments<'a>>,
}

impl<'a> Inflections<'a> {
    pub fn new(dictionary: Full<'a>) -> Self {
        Self {
            dictionary,
            inflections: BTreeMap::new(),
        }
    }

    /// Insert a value into this collection of inflections.
    pub(crate) fn insert(&mut self, inflect: &[Form], inflect2: &[Form], word: Fragments<'a>) {
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
            if c.form.contains(Form::Honorific) {
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
    pub fn get(&self, inflection: Inflection) -> Option<&Fragments<'a>> {
        self.inflections.get(&inflection)
    }

    /// Iterate over all inflections.
    pub fn iter(&self) -> impl Iterator<Item = (&Inflection, &Fragments<'a>)> + '_ {
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
        this: &BTreeMap<Inflection, Fragments<'_>>,
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
    ) -> BTreeMap<Inflection, Fragments<'_>> {
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
