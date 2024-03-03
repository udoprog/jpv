use fixed_map::Key;
use musli::{Decode, Encode};
use musli_zerocopy::{Visit, ZeroCopy};
use serde::{Deserialize, Serialize};

macro_rules! pick {
    ($entity:literal,) => {
        $entity
    };

    ($entity:literal, $parse:literal) => {
        $parse
    };
}

macro_rules! entity {
    (
        $enum_name:ident, $test:ident,

        $(
            $(#[$($meta:meta)*])*
            $vis:vis enum $name:ident {
                $(<$variant:ident $entity:literal $(($parse:literal))? $doc:literal>)*
            }
        )*
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum $enum_name {
            $(
                $name($name),
            )*
        }

        impl $enum_name {
            #[allow(unused)]
            pub(crate) fn parse_keyword(string: &str) -> Option<$enum_name> {
                match string {
                    $(
                        $(pick!($entity, $($parse)*) => Some($enum_name::$name($name::$variant)),)*
                    )*
                    _ => None,
                }
            }
        }

        #[test]
        fn $test() {
            $(
                $(
                    assert_eq!($enum_name::parse_keyword(pick!($entity, $($parse)*)), Some($enum_name::$name($name::$variant)), "Failed to parse `{}`", pick!($entity, $($parse)*));
                )*
            )*
        }

        $(
            $(#[$($meta)*])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Key, ZeroCopy, Visit)]
            #[key(bitset)]
            #[repr(u8)]
            #[non_exhaustive]
            $vis enum $name {
                $(
                    #[doc = $doc]
                    #[serde(rename = $entity)]
                    $variant,
                )*
            }

            impl $name {
                $vis const VALUES: &'static [$name] = &[
                    $($name::$variant,)*
                ];

                $vis fn variant(&self) -> &str {
                    match self {
                        $($name::$variant => stringify!($variant),)*
                    }
                }

                $vis fn ident(&self) -> &str {
                    match self {
                        $($name::$variant => $entity,)*
                    }
                }

                $vis fn help(&self) -> &'static str {
                    match self {
                        $($name::$variant => $doc,)*
                    }
                }

                $vis fn parse(string: &str) -> Option<$name> {
                    match string {
                        $(concat!("&", $entity, ";") => Some($name::$variant),)*
                        _ => None,
                    }
                }

                $vis fn parse_keyword(string: &str) -> Option<$name> {
                    match string {
                        $($entity => Some($name::$variant),)*
                        _ => None,
                    }
                }
            }
        )*
    }
}

entity! {
    Entity, test_entity,

    #[derive(Encode, Decode, Serialize, Deserialize)]
    pub enum Miscellaneous {
        <Abbreviation "abbr" "abbreviation">
        <Archaic "arch" "archaic">
        <Character "char" "character">
        <Children "chn" "children's language">
        <Colloquial "col" "colloquial">
        <Company "company" "company name">
        <Creature "creat" "creature">
        <Dated "dated" "dated term">
        <Deity "dei" "deity">
        <Derogatory "derog" "derogatory">
        <Document "doc" "document">
        <Euphemistic "euph" "euphemistic">
        <Event "ev" "event">
        <Familiar "fam" "familiar language">
        <Feminine "fem" "female term or language">
        <Fict "fict" "fiction">
        <Form "form" "formal or literary term">
        <Given "given" "given name or forename, gender not specified">
        <Group "group" "group">
        <Historical "hist" "historical term">
        <Honorific "hon" "honorific or respectful (sonkeigo) language">
        <Humble "hum" "humble (kenjougo) language">
        <Idiomatic "id" "idiomatic expression">
        <Jocular "joc" "jocular, humorous term">
        <Legend "leg" "legend">
        <MangaSlang "m-sl" "manga slang">
        <Male "male" "male term or language">
        <Mythology "myth" "mythology">
        <NetSlang "net-sl" "Internet slang">
        <Object "obj" "object">
        <Obsolete "obs" "obsolete term">
        <OnMim "on-mim" "onomatopoeic or mimetic word">
        <Organization "organization" "organization name">
        <Other "oth" "other">
        <Person "person" "full name of a particular person">
        <Place "place" "place name">
        <Poetic "poet" "poetical term">
        <Polite "pol" "polite (teineigo) language">
        <Product "product" "product name">
        <Proverb "proverb" "proverb">
        <Quote "quote" "quotation">
        <Rare "rare" "rare term">
        <Relig "relig" "religion">
        <Sens "sens" "sensitive">
        <Service "serv" "service">
        <Ship "ship" "ship name">
        <Slang "sl" "slang">
        <Station "station" "railway station">
        <Surname "surname" "family or surname">
        <UsuallyKana "uk" "word usually written using kana alone">
        <Unclassified "unclass" "unclassified name">
        <Vulgar "vulg" "vulgar expression or word">
        <Work "work" "work of art, literature, music, etc. name">
        <X "X" "rude or X-rated term (not displayed in educational software)">
        <Yojijukugo "yoji" "yojijukugo">
    }

    #[derive(Encode, Decode, Serialize, Deserialize)]
    pub enum PartOfSpeech {
        <AdjectiveF "adj-f" "noun or verb acting prenominally">
        <AdjectiveI "adj-i" "adjective (keiyoushi)">
        <AdjectiveIx "adj-ix" "adjective (keiyoushi) - yoi/ii class">
        <AdjectiveKari "adj-kari" "'kari' adjective (archaic)">
        <AdjectiveKu "adj-ku" "'ku' adjective (archaic)">
        <AdjectiveNa "adj-na" "adjectival nouns or quasi-adjectives (keiyodoshi)">
        <AdjectiveNari "adj-nari" "archaic/formal form of na-adjective">
        <AdjectiveNo "adj-no" "nouns which may take the genitive case particle 'no'">
        <AdjectivePn "adj-pn" "pre-noun adjectival (rentaishi)">
        <AdjectiveShiku "adj-shiku" "'shiku' adjective (archaic)">
        <AdjectiveT "adj-t" "'taru' adjective">
        <Adverb "adv" "adverb (fukushi)">
        <AdverbTo "adv-to" "adverb taking the 'to' particle">
        <Auxiliary "aux" "auxiliary">
        <AuxiliaryAdjective "aux-adj" "auxiliary adjective">
        <AuxiliaryVerb "aux-v" "auxiliary verb">
        <Conjunction "conj" "conjunction">
        <Copular "cop" "copula">
        <Counter "ctr" "counter">
        <Expression "exp" "expressions (phrases, clauses, etc.)">
        <Interjection "int" "interjection (kandoushi)">
        <Noun "n" "noun (common) (futsuumeishi)">
        <NounAdverbial "n-adv" "adverbial noun (fukushitekimeishi)">
        <NounProper "n-pr" "proper noun">
        <NounPrefix "n-pref" "noun, used as a prefix">
        <NounSuffix "n-suf" "noun, used as a suffix">
        <NounTemporal "n-t" "noun (temporal) (jisoumeishi)">
        <Numeric "num" "numeric">
        <Pronoun "pn" "pronoun">
        <Prefix "pref" "prefix">
        <Particle "prt" "particle">
        <Suffix "suf" "suffix">
        <Unclassified "unc" "unclassified">
        <VerbUnspecified "v-unspec" "verb unspecified">
        <VerbIchidan "v1" "Ichidan verb">
        <VerbIchidanS "v1-s" "Ichidan verb - kureru special class">
        <VerbNidanAS "v2a-s" "Nidan verb with 'u' ending (archaic)">
        <VerbNidanBK "v2b-k" "Nidan verb (upper class) with 'bu' ending (archaic)">
        <VerbNidanBS "v2b-s" "Nidan verb (lower class) with 'bu' ending (archaic)">
        <VerbNidanDK "v2d-k" "Nidan verb (upper class) with 'dzu' ending (archaic)">
        <VerbNidanDS "v2d-s" "Nidan verb (lower class) with 'dzu' ending (archaic)">
        <VerbNidanGK "v2g-k" "Nidan verb (upper class) with 'gu' ending (archaic)">
        <VerbNidanGS "v2g-s" "Nidan verb (lower class) with 'gu' ending (archaic)">
        <VerbNidanHK "v2h-k" "Nidan verb (upper class) with 'hu/fu' ending (archaic)">
        <VerbNidanHS "v2h-s" "Nidan verb (lower class) with 'hu/fu' ending (archaic)">
        <VerbNidanKK "v2k-k" "Nidan verb (upper class) with 'ku' ending (archaic)">
        <VerbNidanKS "v2k-s" "Nidan verb (lower class) with 'ku' ending (archaic)">
        <VerbNidanMK "v2m-k" "Nidan verb (upper class) with 'mu' ending (archaic)">
        <VerbNidanMS "v2m-s" "Nidan verb (lower class) with 'mu' ending (archaic)">
        <VerbNidanNS "v2n-s" "Nidan verb (lower class) with 'nu' ending (archaic)">
        <VerbNidanRK "v2r-k" "Nidan verb (upper class) with 'ru' ending (archaic)">
        <VerbNidanRS "v2r-s" "Nidan verb (lower class) with 'ru' ending (archaic)">
        <VerbNidanSS "v2s-s" "Nidan verb (lower class) with 'su' ending (archaic)">
        <VerbNidanTK "v2t-k" "Nidan verb (upper class) with 'tsu' ending (archaic)">
        <VerbNidanTS "v2t-s" "Nidan verb (lower class) with 'tsu' ending (archaic)">
        <VerbNidanWS "v2w-s" "Nidan verb (lower class) with 'u' ending and 'we' conjugation (archaic)">
        <VerbNidanYK "v2y-k" "Nidan verb (upper class) with 'yu' ending (archaic)">
        <VerbNidanYS "v2y-s" "Nidan verb (lower class) with 'yu' ending (archaic)">
        <VerbNidanZS "v2z-s" "Nidan verb (lower class) with 'zu' ending (archaic)">
        <VerbYodanB "v4b" "Yodan verb with 'bu' ending (archaic)">
        <VerbYodanG "v4g" "Yodan verb with 'gu' ending (archaic)">
        <VerbYodanH "v4h" "Yodan verb with 'hu/fu' ending (archaic)">
        <VerbYodanK "v4k" "Yodan verb with 'ku' ending (archaic)">
        <VerbYodanM "v4m" "Yodan verb with 'mu' ending (archaic)">
        <VerbYodanN "v4n" "Yodan verb with 'nu' ending (archaic)">
        <VerbYodanR "v4r" "Yodan verb with 'ru' ending (archaic)">
        <VerbYodanS "v4s" "Yodan verb with 'su' ending (archaic)">
        <VerbYodanT "v4t" "Yodan verb with 'tsu' ending (archaic)">
        <VerbGodanAru "v5aru" "Godan verb - -aru special class">
        <VerbGodanB "v5b" "Godan verb with 'bu' ending">
        <VerbGodanG "v5g" "Godan verb with 'gu' ending">
        <VerbGodanK "v5k" "Godan verb with 'ku' ending">
        <VerbGodanKS "v5k-s" "Godan verb - Iku/Yuku special class">
        <VerbGodanM "v5m" "Godan verb with 'mu' ending">
        <VerbGodanN "v5n" "Godan verb with 'nu' ending">
        <VerbGodanR "v5r" "Godan verb with 'ru' ending">
        <VerbGodanRI "v5r-i" "Godan verb with 'ru' ending (irregular verb)">
        <VerbGodanS "v5s" "Godan verb with 'su' ending">
        <VerbGodanT "v5t" "Godan verb with 'tsu' ending">
        <VerbGodanU "v5u" "Godan verb with 'u' ending">
        <VerbGodanUS "v5u-s" "Godan verb with 'u' ending (special class)">
        <VerbGodanUru "v5uru" "Godan verb - Uru old class verb (old form of Eru)">
        <VerbIntransitive "vi" "intransitive verb">
        <VerbKuru "vk" "Kuru verb - special class">
        <VerbNu "vn" "irregular nu verb">
        <VerbRu "vr" "irregular ru verb, plain form ends with -ri">
        <VerbSuru "vs" "noun or participle which takes the aux. verb suru">
        <VerbSuC "vs-c" "su verb - precursor to the modern suru">
        <VerbSuruIncluded "vs-i" "suru verb - included">
        <VerbSuruSpecial "vs-s" "suru verb - special class">
        <VerbTransitive "vt" "transitive verb">
        <VerbZuru "vz" "Ichidan verb - zuru verb (alternative form of -jiru verbs)">
    }

    #[derive(Encode, Decode, Serialize, Deserialize)]
    pub enum KanjiInfo {
        <Ateji "ateji" "ateji (phonetic) reading">
        <IrregularKana "ik" "word containing irregular kana usage">
        <IrregularKanji "iK" "word containing irregular kanji usage">
        <IrregularOkurigana "io" "irregular okurigana usage">
        <OutdatedKanji "oK" "word containing out-dated kanji or kanji usage">
        <RareKanji "rK" "rarely-used kanji form">
        <SearchOnlyKanji "sK" "search-only kanji form">
    }

    #[derive(Encode, Decode, Serialize, Deserialize)]
    pub enum ReadingInfo {
        <Gikun "gikun" "gikun (meaning as reading) or jukujikun (special kanji reading)">
        <IrregularKana "ik" ("rik") "word containing irregular kana usage">
        <ObsoleteKana "ok" "out-dated or obsolete kana usage">
        <SearchOnlyKana "sk" "search-only kana form">
        <RareKana "rk" "rarely used kana form">
    }

    #[derive(Encode, Decode, Serialize, Deserialize)]
    pub enum Dialect {
        <Brazilian "bra" "Brazilian">
        <HokkaidoBen "hob" "Hokkaido-ben">
        <KansaiBen "ksb" "Kansai-ben">
        <KantouBen "ktb" "Kantou-ben">
        <KyotoBen "kyb" "Kyoto-ben">
        <KyuushuuBen "kyu" "Kyuushuu-ben">
        <NaganoBen "nab" "Nagano-ben">
        <OsakaBen "osb" "Osaka-ben">
        <RyuukyuuBen "rkb" "Ryuukyuu-ben">
        <TouhokuBen "thb" "Touhoku-ben">
        <TosaBen "tsb" "Tosa-ben">
        <TsugaruBen "tsug" "Tsugaru-ben">
    }

    #[derive(Encode, Decode, Serialize, Deserialize)]
    pub enum Field {
        <Agriculture "agric" "agriculture">
        <Anatomy "anat" "anatomy">
        <Archeology "archeol" "archeology">
        <Architecture "archit" "architecture">
        <Art "art" "art, aesthetics">
        <Astronomy "astron" "astronomy">
        <AudioVisual "audvid" "audiovisual">
        <Aviatation "aviat" "aviation">
        <Baseball "baseb" "baseball">
        <Biochemistry "biochem" "biochemistry">
        <Biology "biol" "biology">
        <Botany "bot" "botany">
        <Boxing "boxing" "boxing">
        <Buddh "Buddh" "Buddhism">
        <Business "bus" "business">
        <Cards "cards" "card games">
        <Chemistry "chem" "chemistry">
        <Christianity "Christn" "Christianity">
        <CivilEngineering "civeng" "civil engineering">
        <ChineseMythology "chmyth" "Chinese mythology">
        <Clothing "cloth" "clothing">
        <Computing "comp" "computing">
        <Crystallography "cryst" "crystallography">
        <Dentistry "dent" "dentistry">
        <Ecology "ecol" "ecology">
        <Economy "econ" "economics">
        <Electricity "elec" "electricity, elec. eng.">
        <Electronics "electr" "electronics">
        <Embryology "embryo" "embryology">
        <Engineering "engr" "engineering">
        <Entomology "ent" "entomology">
        <Film "film" "film">
        <Finc "finc" "finance">
        <Fish "fish" "fishing">
        <Food "food" "food, cooking">
        <Gardening "gardn" "gardening, horticulture">
        <Genetics "genet" "genetics">
        <Geography "geogr" "geography">
        <Geology "geol" "geology">
        <Geometry "geom" "geometry">
        <Go "go" "go (game)">
        <Golf "golf" "golf">
        <Grammar "gramm" "grammar">
        <GreekMythology "grmyth" "Greek mythology">
        <Hanafuda "hanaf" "hanafuda">
        <Horse "horse" "horse racing">
        <Internet "internet" "internet">
        <JapaneseMythology "jpmyth" "Japanese mythology">
        <Kabuki "kabuki" "kabuki">
        <Law "law" "law">
        <Ling "ling" "linguistics">
        <Logic "logic" "logic">
        <MartialArts "MA" "martial arts">
        <Mahjong "mahj" "mahjong">
        <Manga "manga" "manga">
        <Mathematics "math" "mathematics">
        <MechanicalEnginering "mech" "mechanical engineering">
        <Medicine "med" "medicine">
        <Meteorology "met" "meteorology">
        <Military "mil" "military">
        <Mining "mining" "mining">
        <Motorsport "motor" "motorsport">
        <Music "music" "music">
        <Noh "noh" "noh">
        <Ornithology "ornith" "ornithology">
        <Paleontology "paleo" "paleontology">
        <Pathology "pathol" "pathology">
        <Pharmacology "pharm" "pharmacology">
        <Philosophy "phil" "philosophy">
        <Photo "photo" "photography">
        <Physics "physics" "physics">
        <Physiol "physiol" "physiology">
        <Politics "politics" "politics">
        <Print "print" "printing">
        <ProwRes "prowres" "professional wrestling">
        <Psychatry "psy" "psychiatry">
        <Psyanal "psyanal" "psychoanalysis">
        <Psychology "psych" "psychology">
        <Railway "rail" "railway">
        <RomanMythology "rommyth" "Roman mythology">
        <Shinto "Shinto" "Shinto">
        <Shogi "shogi" "shogi">
        <Skiing "ski" "skiing">
        <Sports "sports" "sports">
        <Statistics "stat" "statistics">
        <StockMarket "stockm" "stock market">
        <Sumo "sumo" "sumo">
        <Surgery "surg" "surgery">
        <Telecommunications "telec" "telecommunications">
        <Trademark "tradem" "trademark">
        <Tv "tv" "television">
        <VideoGames "vidg" "video games">
        <Zoology "zool" "zoology">
    }
}

entity! {
    NameEntity, test_name_entity,

    #[derive(Encode, Decode, Serialize, Deserialize)]
    pub enum NameType {
        <Character "char" "character">
        <Company "company" "company name">
        <Creature "creat" "creature">
        <Deity "dei" "deity">
        <Document "doc" "document">
        <Event "ev" "event">
        <Feminine "fem" "female given name or forename">
        <Fiction "fict" "fiction">
        <Given "given" "given name or forename, gender not specified">
        <Group "group" "group">
        <Legend "leg" "legend">
        <Masculine "masc" "male given name or forename">
        <Mythology "myth" "mythology">
        <Object "obj" "object">
        <Organization "organization" "organization name">
        <Other "oth" "other">
        <Person "person" "full name of a particular person">
        <Place "place" "place name">
        <Product "product" "product name">
        <Religion "relig" "religion">
        <Service "serv" "service">
        <Ship "ship" "ship name">
        <Station "station" "railway station">
        <Surname "surname" "family or surname">
        <Unclassified "unclass" "unclassified name">
        <Work "work" "work of art, literature, music, etc. name">
    }
}
