use std::fs;

use anyhow::Result;

use crate::config::Config;
use crate::reporter::EmptyReporter;
use crate::token::Token;

use super::{build, Database, Input, Location};

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
