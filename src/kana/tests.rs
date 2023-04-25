use super::Furigana;

#[test]
fn test_mixed_furigana() {
    let furigana = Furigana::borrowed("私はお金がない", "わたしはおかねがない");
    assert_eq!(furigana.to_string(), "私[わたし]はお金[かね]がない");
}
