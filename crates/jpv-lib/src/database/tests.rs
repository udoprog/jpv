use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;

use crate::config::Config;
use crate::inflection::{Inflection, Reading};
use crate::reporter::EmptyReporter;
use crate::token::Token;

use super::{build, Database, InflectionData, Input, Location, PhraseIndex, Source};

const KANJIDIC2: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<kanjidic2>
<header>
<file_version>4</file_version>
<database_version>test</database_version>
<date_of_creation>test</date_of_creation>
</header>
<character>
<literal>亜</literal>
<codepoint>
<cp_value cp_type="ucs">4e9c</cp_value>
</codepoint>
<radical>
<rad_value rad_type="classical">7</rad_value>
</radical>
<misc>
<grade>8</grade>
<stroke_count>7</stroke_count>
</misc>
<dic_number>
<dic_ref dr_type="nelson_c">43</dic_ref>
</dic_number>
<query_code>
<q_code qc_type="skip">4-7-1</q_code>
</query_code>
<reading_meaning>
<rmgroup>
<reading r_type="ja_on">ア</reading>
<meaning>Asia</meaning>
<meaning m_lang="fr">Asie</meaning>
</rmgroup>
</reading_meaning>
</character>
</kanjidic2>
"#;

/// Index entries with absent optional fields must decode back from a stored
/// index.
#[test]
fn kanjidic2_round_trip() -> Result<()> {
    let buf = build(
        &EmptyReporter,
        &Token::default(),
        "kanjidic2",
        Input::Kanjidic2(KANJIDIC2),
    )?;

    let path = std::env::temp_dir().join(format!(
        "jpv-lib-test-{}-kanjidic2.index",
        std::process::id()
    ));
    fs::write(&path, buf.as_slice())?;

    let result = (|| {
        let data = crate::data::open(&path)?;
        let db = Database::open(
            [(data, Location::Path(path.clone().into()))],
            &Config::default(),
        )?;

        let c = db.literal_to_kanji("亜")?.expect("missing character");
        assert_eq!(c.literal, "亜");
        assert_eq!(c.misc.grade, Some(8));
        assert_eq!(c.misc.freq, None);
        assert_eq!(c.misc.jlpt, None);
        assert_eq!(c.dictionary_references[0].volume, None);
        assert_eq!(c.query_codes[0].skip_misclass, None);
        assert_eq!(c.meanings[0].lang, None);
        assert_eq!(c.meanings[1].lang, Some("fr"));
        assert_eq!(c.readings[0].text, "ア");
        Ok(())
    })();

    fs::remove_file(&path)?;
    result
}

const JMDICT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<JMdict>
<entry>
<ent_seq>1358280</ent_seq>
<k_ele><keb>食べる</keb></k_ele>
<r_ele><reb>たべる</reb></r_ele>
<sense><pos>&v1;</pos><gloss>to eat</gloss></sense>
</entry>
<entry>
<ent_seq>1578850</ent_seq>
<k_ele><keb>行く</keb></k_ele>
<r_ele><reb>いく</reb></r_ele>
<sense><pos>&v5k-s;</pos><gloss>to go</gloss></sense>
</entry>
<entry>
<ent_seq>1398780</ent_seq>
<k_ele><keb>走る</keb></k_ele>
<r_ele><reb>はしる</reb></r_ele>
<sense><pos>&v5r;</pos><gloss>to run</gloss></sense>
</entry>
<entry>
<ent_seq>1547720</ent_seq>
<k_ele><keb>来る</keb></k_ele>
<r_ele><reb>くる</reb></r_ele>
<sense><pos>&vk;</pos><gloss>to come</gloss></sense>
</entry>
<entry>
<ent_seq>1157170</ent_seq>
<r_ele><reb>する</reb></r_ele>
<sense><pos>&vs-i;</pos><gloss>to do</gloss></sense>
</entry>
<entry>
<ent_seq>1289400</ent_seq>
<k_ele><keb>高い</keb></k_ele>
<r_ele><reb>たかい</reb></r_ele>
<sense><pos>&adj-i;</pos><gloss>high</gloss></sense>
</entry>
<entry>
<ent_seq>2820690</ent_seq>
<k_ele><keb>良い</keb></k_ele>
<r_ele><reb>いい</reb></r_ele>
<sense><pos>&adj-ix;</pos><gloss>good</gloss></sense>
</entry>
<entry>
<ent_seq>1375710</ent_seq>
<k_ele><keb>静か</keb></k_ele>
<r_ele><reb>しずか</reb></r_ele>
<sense><pos>&adj-na;</pos><gloss>quiet</gloss></sense>
</entry>
<entry>
<ent_seq>1006680</ent_seq>
<r_ele><reb>サボる</reb></r_ele>
<sense><pos>&v5r;</pos><gloss>to skip class</gloss></sense>
</entry>
<entry>
<ent_seq>1467640</ent_seq>
<k_ele><keb>猫</keb></k_ele>
<r_ele><reb>ねこ</reb></r_ele>
<sense><pos>&n;</pos><gloss>cat</gloss></sense>
</entry>
</JMdict>
"#;

struct TempIndex(PathBuf);

impl Drop for TempIndex {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn open_jmdict() -> Result<(TempIndex, Database)> {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    let buf = build(
        &EmptyReporter,
        &Token::default(),
        "jmdict",
        Input::Jmdict(JMDICT),
    )?;

    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "jpv-lib-test-{}-{id}-jmdict.index",
        std::process::id()
    ));
    fs::write(&path, buf.as_slice())?;

    let temp = TempIndex(path.clone());
    let data = crate::data::open(&path)?;
    let db = Database::open([(data, Location::Path(path.into()))], &Config::default())?;
    Ok((temp, db))
}

fn inflection_source(reading: Reading, inflection: Inflection) -> Source {
    Source::Inflection {
        data: InflectionData {
            reading,
            inflection,
        },
    }
}

#[test]
fn search_dictionary_forms_stay_phrase_sources() -> Result<()> {
    let (_temp, db) = open_jmdict()?;

    for (query, expected_seq, expected_index) in [
        ("食べる", 1358280, PhraseIndex::Kanji),
        ("たべる", 1358280, PhraseIndex::Hiragana),
        ("タベル", 1358280, PhraseIndex::Katakana),
        ("taberu", 1358280, PhraseIndex::Romanized),
        ("高い", 1289400, PhraseIndex::Kanji),
        ("静か", 1375710, PhraseIndex::Kanji),
    ] {
        let result = db.search(query)?;
        assert_eq!(result.phrases.len(), 1, "query: {query}");
        let (key, entry) = &result.phrases[0];
        assert_eq!(entry.sequence, expected_seq, "query: {query}");
        assert!(
            key.sources.contains(&Source::Phrase {
                index: expected_index,
            }),
            "expected Phrase({expected_index:?}) for {query}, got {:?}",
            key.sources
        );
        assert!(
            !key.sources.iter().any(Source::is_inflection),
            "unexpected inflection source for dictionary form {query}: {:?}",
            key.sources
        );
    }

    Ok(())
}

#[test]
fn search_stemmed_inflections() -> Result<()> {
    let (_temp, db) = open_jmdict()?;
    let reading_0_0 = Reading {
        kanji: 0,
        reading: 0,
    };
    let kana_only = Reading {
        kanji: u8::MAX,
        reading: 0,
    };

    let cases: &[(&str, u64, Reading, Inflection)] = &[
        // Kanji + okurigana verbs
        ("食べた", 1358280, reading_0_0, inflect!(Past)),
        (
            "食べていた",
            1358280,
            reading_0_0,
            inflect!(Te, TeIru, Past),
        ),
        ("行った", 1578850, reading_0_0, inflect!(Past)),
        (
            "走らなかった",
            1398780,
            reading_0_0,
            inflect!(Negative, Past),
        ),
        ("来た", 1547720, reading_0_0, inflect!(Past)),
        // Hiragana verbs
        ("たべた", 1358280, reading_0_0, inflect!(Past)),
        (
            "たべていた",
            1358280,
            reading_0_0,
            inflect!(Te, TeIru, Past),
        ),
        ("いった", 1578850, reading_0_0, inflect!(Past)),
        ("きた", 1547720, reading_0_0, inflect!(Past)),
        ("した", 1157170, kana_only, inflect!(Past)),
        // Katakana & romaji verbs
        ("タベタ", 1358280, reading_0_0, inflect!(Past)),
        ("イッタ", 1578850, reading_0_0, inflect!(Past)),
        ("tabeta", 1358280, reading_0_0, inflect!(Past)),
        ("itta", 1578850, reading_0_0, inflect!(Past)),
        // Katakana-only verb
        ("サボった", 1006680, kana_only, inflect!(Past)),
        // Adjectives (i, ix, na)
        ("高かった", 1289400, reading_0_0, inflect!(Past)),
        ("たかかった", 1289400, reading_0_0, inflect!(Past)),
        ("タカカッタ", 1289400, reading_0_0, inflect!(Past)),
        ("takakatta", 1289400, reading_0_0, inflect!(Past)),
        ("よかった", 2820690, reading_0_0, inflect!(Past)),
        ("yokatta", 2820690, reading_0_0, inflect!(Past)),
        ("静かだった", 1375710, reading_0_0, inflect!(Past)),
        ("静かだ", 1375710, reading_0_0, Inflection::default()),
    ];

    for &(query, expected_seq, expected_reading, expected_inflection) in cases {
        let result = db.search(query)?;
        let expected_source = inflection_source(expected_reading, expected_inflection);
        let found = result
            .phrases
            .iter()
            .find(|(_, entry)| entry.sequence == expected_seq);
        let Some((key, _)) = found else {
            panic!("expected sequence {expected_seq} for query {query}");
        };
        assert!(
            key.sources.contains(&expected_source),
            "expected {expected_source:?} for {query}, got {:?}",
            key.sources
        );
    }

    // Invalid conjugations and non-conjugable nouns must not match.
    for invalid in ["行いた", "走た", "猫た"] {
        let result = db.search(invalid)?;
        assert!(
            result.phrases.is_empty(),
            "expected no matches for {invalid}, got {}",
            result.phrases.len()
        );
    }

    Ok(())
}

#[test]
fn analyze_matches_dictionary_and_inflected_prefixes() -> Result<()> {
    let (_temp, db) = open_jmdict()?;

    let verb = db.analyze("食べていたよ", 0)?;
    let verb_texts: Vec<&str> = verb.iter().map(|&(_, text)| text).collect();
    assert!(
        verb_texts.contains(&"食べていた"),
        "expected 食べていた in {verb_texts:?}"
    );
    assert!(
        verb_texts.contains(&"食べて"),
        "expected 食べて in {verb_texts:?}"
    );

    let dict = db.analyze("食べるよ", 0)?;
    let dict_texts: Vec<&str> = dict.iter().map(|&(_, text)| text).collect();
    assert!(
        dict_texts.contains(&"食べる"),
        "expected 食べる in {dict_texts:?}"
    );

    let adj = db.analyze("高かったよ", 0)?;
    let adj_texts: Vec<&str> = adj.iter().map(|&(_, text)| text).collect();
    assert!(
        adj_texts.contains(&"高かった"),
        "expected 高かった in {adj_texts:?}"
    );

    let na_adj = db.analyze("静かだったよ", 0)?;
    let na_texts: Vec<&str> = na_adj.iter().map(|&(_, text)| text).collect();
    assert!(
        na_texts.contains(&"静かだった"),
        "expected 静かだった in {na_texts:?}"
    );
    assert!(na_texts.contains(&"静か"), "expected 静か in {na_texts:?}");

    Ok(())
}
