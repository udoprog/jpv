use std::ops::Range;

const NUL: char = '\0';

/// Helper to analyze a search query.
#[derive(Default)]
pub struct SearchQuery<'a> {
    pub phrases: Vec<&'a str>,
    pub phrase_ranges: Vec<Range<usize>>,
    pub entities: Vec<&'a str>,
}

/// Parse an input.
pub fn parse(input: &str) -> SearchQuery<'_> {
    SearchParser::new(input).parse()
}

struct SearchParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> SearchParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or(NUL)
    }

    fn step(&mut self) -> char {
        let Some(c) = self.input[self.pos..].chars().next() else {
            return NUL;
        };

        self.pos += c.len_utf8();
        c
    }

    fn ws(&mut self) {
        while self.peek().is_whitespace() {
            self.step();
        }
    }

    fn ident(&mut self) -> &'a str {
        fn is_ident(c: char) -> bool {
            c.is_ascii_alphanumeric() || c == '-'
        }

        let start = self.pos;

        while is_ident(self.peek()) {
            self.step();
        }

        &self.input[start..self.pos]
    }

    fn parse(&mut self) -> SearchQuery<'a> {
        let mut query = SearchQuery::default();

        let mut start = None;
        let mut end = self.pos;

        while self.pos < self.input.len() {
            end = self.pos;

            self.ws();

            match self.peek() {
                NUL => {
                    continue;
                }
                '#' => {
                    if let Some(start) = start.take() {
                        query.phrase_ranges.push(start..end);
                        query.phrases.push(&self.input[start..end]);
                    }

                    self.step();
                    query.entities.push(self.ident());
                }
                ',' | '、' | '.' | '。' => {
                    if let Some(start) = start.take() {
                        query.phrase_ranges.push(start..end);
                        query.phrases.push(&self.input[start..end]);
                    }

                    self.step();
                }
                _ => {
                    if start.is_none() {
                        start = Some(self.pos);
                    }

                    self.step();
                    end = self.pos;
                }
            }
        }

        if let Some(start) = start.take() {
            query.phrase_ranges.push(start..end);
            query.phrases.push(&self.input[start..end]);
        }

        query
    }
}

#[test]
fn test_parse() {
    let mut parser =
        SearchParser::new("\t\thello world #v5s first tail phrase*, , ,,, second tail phrase\n\n");
    let query = parser.parse();

    assert_eq!(query.entities.len(), 1);
    assert_eq!(query.entities[0], "v5s");
    assert_eq!(query.phrases.len(), 3);
    assert_eq!(query.phrases[0], "hello world");
    assert_eq!(query.phrases[1], "first tail phrase*");
    assert_eq!(query.phrases[2], "second tail phrase");
}
