use crate::kana;

/// Construct an iterator over morea in the given input.
pub fn iter(input: &str) -> Morae<'_> {
    Morae { input }
}

/// Iterate over morae.
pub struct Morae<'a> {
    input: &'a str,
}

impl<'a> Iterator for Morae<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut it = self.input.chars();

        let a = it.next()?;
        let b = it
            .next()
            .map(|c| (c, kana::is_hiragana_lower(c) || kana::is_katakana_lower(c)));

        let head = match (kana::is_hiragana_upper(a) || kana::is_katakana_upper(a), b) {
            (true, Some((b, true))) => {
                let end = a.len_utf8() + b.len_utf8();
                let (head, tail) = self.input.split_at(end);
                self.input = tail;
                head
            }
            _ => {
                let end = a.len_utf8();
                let (head, tail) = self.input.split_at(end);
                self.input = tail;
                head
            }
        };

        Some(head)
    }
}

#[test]
fn count_morae() {
    let input = "ひらがな";
    let morae: Vec<_> = iter(input).collect();
    assert_eq!(morae, vec!["ひ", "ら", "が", "な"]);

    let input = "とうきょう";
    let morae: Vec<_> = iter(input).collect();
    assert_eq!(morae, vec!["と", "う", "きょ", "う"]);

    let input = "モーラ";
    let morae: Vec<_> = iter(input).collect();
    assert_eq!(morae, vec!["モ", "ー", "ラ"]);
}
